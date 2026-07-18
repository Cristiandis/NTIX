using Lua;
using Lua.Standard;
using NTIX.Core.Models;

namespace NTIX.Core.Config;

public static class ConfigLoader
{
    private static readonly string[] PackageListKeys = { "winget", "chocolatey", "scoop" };
    public static NTIXConfig Load(string configPath)
    {
        if (!File.Exists(configPath))
            throw new FileNotFoundException($"Config file not found: {configPath}");

        var script = File.ReadAllText(configPath);
        return LoadFromString(script, configPath);
    }

    public static NTIXConfig LoadFromString(string luaScript, string configPath)
    {
        using var state = LuaState.Create();
        state.OpenStandardLibraries();

        var fullConfigPath = Path.GetFullPath(configPath);
        var rootDir = Path.GetDirectoryName(fullConfigPath) ?? "";

        if (!string.IsNullOrEmpty(rootDir))
        {
            var pkg = state.Environment["package"].Read<LuaTable>();
            pkg["path"] = pkg["path"].Read<string>() + ";" + rootDir.Replace('\\', '/') + "/?.lua";
        }

        var globalOptions = new LuaTable();
        var globalPkgs = new LuaTable();
        foreach (var key in PackageListKeys)
            globalPkgs[key] = new LuaTable();

        state.Environment["options"] = globalOptions;
        state.Environment["pkgs"] = globalPkgs;

        RegisterImportFunction(state, fullConfigPath, globalOptions, globalPkgs);

        try
        {
            var results = state.DoStringAsync(luaScript).GetAwaiter().GetResult();
            return ParseConfig(results[0], configPath);
        }
        catch (LuaCompileException ex)
        {
            throw new InvalidOperationException($"Lua syntax error: {ex.Message}");
        }
        catch (LuaRuntimeException ex)
        {
            throw new InvalidOperationException($"Lua runtime error: {ex.Message}");
        }
        catch (OperationCanceledException)
        {
            throw;
        }
    }
    private static void RegisterImportFunction(
        LuaState state,
        string rootConfigPath,
        LuaTable globalOptions,
        LuaTable globalPkgs)
    {
        var directoryStack = new Stack<string>();
        directoryStack.Push(Path.GetDirectoryName(rootConfigPath) ?? "");

        state.Environment["import"] = new LuaFunction((context, ct) =>
        {
            var arg = context.GetArgument<LuaValue>(0);
            var paths = new List<string>();

            switch (arg.Type)
            {
                case LuaValueType.String:
                    paths.Add(arg.Read<string>());
                    break;
                case LuaValueType.Table:
                    foreach (var kvp in arg.Read<LuaTable>())
                    {
                        if (kvp.Key.Type != LuaValueType.Number) continue;
                        var p = kvp.Value.Read<string>();
                        if (!string.IsNullOrWhiteSpace(p))
                            paths.Add(p);
                    }
                    break;
                default:
                    throw new InvalidOperationException(
                        "import() expects a file path string or an array of file path strings");
            }

            var currentDir = directoryStack.Peek();

            foreach (var relativePath in paths)
            {
                var importPath = Path.GetFullPath(Path.Combine(currentDir, relativePath));

                if (!File.Exists(importPath))
                    throw new FileNotFoundException($"Import file not found: {importPath} (referenced from config)");

                // Let the imported file require() siblings from its own directory.
                var importDir = Path.GetDirectoryName(importPath)?.Replace('\\', '/') ?? "";
                var pkg = state.Environment["package"].Read<LuaTable>();
                pkg["path"] = pkg["path"].Read<string>() + ";" + importDir + "/?.lua";

                var script = File.ReadAllText(importPath);

                directoryStack.Push(Path.GetDirectoryName(importPath) ?? "");
                LuaValue[] importResults;
                try
                {
                    // Nested import() calls made while this script runs merge
                    // into globalOptions/globalPkgs as their own side effect.
                    importResults = state.DoStringAsync(script).GetAwaiter().GetResult();
                }
                finally
                {
                    directoryStack.Pop();
                }

                if (importResults.Length > 0 && importResults[0].Type == LuaValueType.Table)
                {
                    MergeReturnedTable(globalOptions, globalPkgs, importResults[0].Read<LuaTable>());
                }
            }

            return default;
        });
    }

    private static void MergeReturnedTable(LuaTable globalOptions, LuaTable globalPkgs, LuaTable returned)
    {
        if (returned["options"].Type == LuaValueType.Table)
            DeepMergeTable(globalOptions, returned["options"].Read<LuaTable>());

        if (returned["pkgs"].Type == LuaValueType.Table)
        {
            var returnedPkgs = returned["pkgs"].Read<LuaTable>();
            foreach (var key in PackageListKeys)
            {
                if (returnedPkgs[key].Type == LuaValueType.Table)
                    MergePackagesDeduped(GetOrCreateSubTable(globalPkgs, key), returnedPkgs[key].Read<LuaTable>());
            }
        }
    }

    private static LuaTable GetOrCreateSubTable(LuaTable parent, string key)
    {
        if (parent[key].Type == LuaValueType.Table)
            return parent[key].Read<LuaTable>();

        var t = new LuaTable();
        parent[key] = t;
        return t;
    }

    /// <summary>
    /// Recursively merges <paramref name="source"/> into <paramref name="target"/>.
    /// Nested tables are merged key-by-key; scalars overwrite.
    /// </summary>
    private static void DeepMergeTable(LuaTable target, LuaTable source)
    {
        foreach (var kvp in source)
        {
            var key = kvp.Key;
            var value = kvp.Value;

            if (value.Type == LuaValueType.Table && target[key].Type == LuaValueType.Table)
                DeepMergeTable(target[key].Read<LuaTable>(), value.Read<LuaTable>());
            else
                target[key] = value;
        }
    }

    /// <summary>
    /// Merges package arrays into target, deduplicating by id (later wins).
    /// </summary>
    private static void MergePackagesDeduped(LuaTable target, LuaTable source)
    {
        var idToIndex = new Dictionary<string, long>();
        long nextIndex = 1;

        foreach (var kvp in target)
        {
            if (kvp.Key.Type != LuaValueType.Number) continue;
            var idx = (long)kvp.Key.Read<double>();
            nextIndex = Math.Max(nextIndex, idx + 1);

            var existingId = ExtractId(kvp.Value);
            if (existingId != null)
                idToIndex[existingId] = idx;
        }

        foreach (var kvp in source)
        {
            if (kvp.Key.Type != LuaValueType.Number) continue;

            var id = ExtractId(kvp.Value);
            if (id == null) continue;

            if (idToIndex.TryGetValue(id, out var existingIndex))
            {
                target[existingIndex] = kvp.Value;
            }
            else
            {
                target[nextIndex] = kvp.Value;
                idToIndex[id] = nextIndex;
                nextIndex++;
            }
        }
    }

    private static string? ExtractId(LuaValue entry) => entry.Type switch
    {
        LuaValueType.String => entry.Read<string>(),
        LuaValueType.Table when entry.Read<LuaTable>()["id"].Type == LuaValueType.String
            => entry.Read<LuaTable>()["id"].Read<string>(),
        _ => null
    };

    private static NTIXConfig ParseConfig(LuaValue result, string configPath)
    {
        if (result.Type != LuaValueType.Table)
            throw new InvalidOperationException($"Config script must return a table (got {result.Type}): {configPath}");

        var table = result.Read<LuaTable>();

        if (table["options"].Type != LuaValueType.Table)
            throw new InvalidOperationException("Config error: missing top-level 'options' table");

        if (table["pkgs"].Type != LuaValueType.Table)
            throw new InvalidOperationException("Config error: missing top-level 'pkgs' table");

        var options = ReadOptions(table["options"].Read<LuaTable>());
        var pkgs = table["pkgs"].Read<LuaTable>();

        return new NTIXConfig(
            Options: options,
            WingetPackages: ReadPackageList(pkgs, "winget"),
            ChocoPackages: ReadPackageList(pkgs, "chocolatey"),
            ScoopPackages: ReadPackageList(pkgs, "scoop")
        );
    }

    private static NTIXOptions ReadOptions(LuaTable options)
    {
        var winget = new WingetOptions();
        if (options["winget"].Type == LuaValueType.Table)
        {
            var t = options["winget"].Read<LuaTable>();
            winget = new WingetOptions(
                Enable: ReadBool(t["enable"], winget.Enable),
                AcceptAgreements: ReadBool(t["acceptAgreements"], winget.AcceptAgreements),
                Interactive: ReadBool(t["interactive"], winget.Interactive)
            );
        }

        var choco = new ChocoOptions();
        if (options["chocolatey"].Type == LuaValueType.Table)
        {
            var t = options["chocolatey"].Read<LuaTable>();
            choco = new ChocoOptions(
                Enable: ReadBool(t["enable"], choco.Enable),
                Yes: ReadBool(t["yes"], choco.Yes)
            );
        }

        var scoop = new ScoopOptions();
        if (options["scoop"].Type == LuaValueType.Table)
        {
            var t = options["scoop"].Read<LuaTable>();
            var buckets = scoop.Buckets;
            if (t["buckets"].Type == LuaValueType.Table)
            {
                buckets = t["buckets"].Read<LuaTable>()
                    .Where(kvp => kvp.Key.Type == LuaValueType.Number)
                    .Select(kvp => kvp.Value.Read<string>())
                    .ToList();
            }
            scoop = new ScoopOptions(
                Enable: ReadBool(t["enable"], scoop.Enable),
                Buckets: buckets
            );
        }

        return new NTIXOptions(winget, choco, scoop);
    }

    private static bool ReadBool(LuaValue value, bool fallback) =>
        value.Type == LuaValueType.Boolean ? value.Read<bool>() : fallback;

    private static List<PackageEntry> ReadPackageList(LuaTable pkgs, string key)
    {
        var list = new List<PackageEntry>();
        if (pkgs[key].Type != LuaValueType.Table)
            return list;

        foreach (var kvp in pkgs[key].Read<LuaTable>())
        {
            if (kvp.Key.Type != LuaValueType.Number) continue;

            switch (kvp.Value.Type)
            {
                case LuaValueType.String:
                    list.Add(new PackageEntry(kvp.Value.Read<string>()));
                    break;
                case LuaValueType.Table:
                    var entry = kvp.Value.Read<LuaTable>();
                    if (entry["id"].Type != LuaValueType.String)
                        continue;
                    var version = entry["version"].Type == LuaValueType.String
                        ? entry["version"].Read<string>()
                        : null;
                    list.Add(new PackageEntry(entry["id"].Read<string>(), version));
                    break;
            }
        }

        return list;
    }
}