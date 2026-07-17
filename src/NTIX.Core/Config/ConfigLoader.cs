using Lua;
using Lua.Standard;
using NTIX.Core.Models;

namespace NTIX.Core.Config;

public static class ConfigLoader
{
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

        var directory = Path.GetDirectoryName(configPath) ?? "";
        if (!string.IsNullOrEmpty(directory))
        {
            var pkg = state.Environment["package"].Read<LuaTable>();
            var path = pkg["path"].Read<string>();
            var newPath = path + ";" + directory.Replace('\\', '/') + "/?.lua";
            pkg["path"] = newPath;
        }

        state.OpenStandardLibraries();

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

    private static NTIXConfig ParseConfig(LuaValue result, string configPath)
    {
        if (result.Type != LuaValueType.Table)
            throw new InvalidOperationException($"Config must return a table, got {result.Type}");

        var table = result.Read<LuaTable>();

        var optionsVal = table["options"];
        if (optionsVal.Type == LuaValueType.Nil)
            throw new InvalidOperationException("Config error: missing top-level 'options' table");

        var optionsTable = optionsVal.Read<LuaTable>();

        var wingetOptions = ParseWingetOptions(optionsTable);
        var chocoOptions = ParseChocoOptions(optionsTable);
        var scoopOptions = ParseScoopOptions(optionsTable);

        var options = new NTIXOptions(wingetOptions, chocoOptions, scoopOptions);

        var pkgsVal = table["pkgs"];
        if (pkgsVal.Type == LuaValueType.Nil)
            throw new InvalidOperationException("Config error: missing top-level 'pkgs' table");

        var pkgsTable = pkgsVal.Read<LuaTable>();

        var wingetPackages = ExtractPackages(pkgsTable["winget"], "pkgs.winget");
        var chocoPackages = ExtractPackages(pkgsTable["chocolatey"], "pkgs.chocolatey");
        var scoopPackages = ExtractPackages(pkgsTable["scoop"], "pkgs.scoop");

        return new NTIXConfig(options, wingetPackages, chocoPackages, scoopPackages);
    }

    private static WingetOptions ParseWingetOptions(LuaTable optionsTable)
    {
        var wingetVal = optionsTable["winget"];
        if (wingetVal.Type == LuaValueType.Nil)
            return new WingetOptions();

        var table = wingetVal.Read<LuaTable>();

        return new WingetOptions(
            Enable: table.ContainsKey("enable") && table["enable"].Type == LuaValueType.Boolean ? table["enable"].Read<bool>() : false,
            AcceptAgreements: table.ContainsKey("acceptAgreements") && table["acceptAgreements"].Type == LuaValueType.Boolean ? table["acceptAgreements"].Read<bool>() : false,
            Interactive: table.ContainsKey("interactive") && table["interactive"].Type == LuaValueType.Boolean ? table["interactive"].Read<bool>() : false
        );
    }

    private static ChocoOptions ParseChocoOptions(LuaTable optionsTable)
    {
        var chocoVal = optionsTable["chocolatey"];
        if (chocoVal.Type == LuaValueType.Nil)
            return new ChocoOptions();

        var table = chocoVal.Read<LuaTable>();

        static bool HasBool(LuaTable t, string key) => t.ContainsKey(key) && t[key].Type == LuaValueType.Boolean;

        return new ChocoOptions(
            Enable: table.ContainsKey("enable") && table["enable"].Type == LuaValueType.Boolean ? table["enable"].Read<bool>() : false,
            Yes: table.ContainsKey("yes") && table["yes"].Type == LuaValueType.Boolean ? table["yes"].Read<bool>() : false
        );
    }

    private static ScoopOptions ParseScoopOptions(LuaTable optionsTable)
    {
        var scoopVal = optionsTable["scoop"];
        if (scoopVal.Type == LuaValueType.Nil)
            return new ScoopOptions();

        var table = scoopVal.Read<LuaTable>();
        var buckets = new List<string> { "main", "extras", "versions" };
        var bucketsVal = table["buckets"];
        if (bucketsVal.Type == LuaValueType.Table)
        {
            var bucketsList = new List<string>();
            foreach (var kvp in bucketsVal.Read<LuaTable>())
            {
                if (kvp.Value.Type == LuaValueType.String)
                    buckets.Add(kvp.Value.Read<string>());
            }
        }

        return new ScoopOptions(
            Enable: table.ContainsKey("enable") && table["enable"].Type == LuaValueType.Boolean ? table["enable"].Read<bool>() : false,
            Buckets: buckets
        );
    }

    private static List<PackageEntry> ExtractPackages(LuaValue pkgsVal, string sourceName)
    {
        var result = new List<PackageEntry>();

        if (pkgsVal.Type == LuaValueType.Nil || pkgsVal.Type != LuaValueType.Table)
            return result;

        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var kvp in pkgsVal.Read<LuaTable>())
        {
            if (kvp.Key.Type != LuaValueType.Number)
                continue;

            var entry = ParsePackageEntry(kvp.Value, sourceName);
            if (entry == null)
                continue;

            if (seen.Contains(entry.Id))
            {
                Console.Error.WriteLine($"[warn] duplicate package '{entry.Id}' in {sourceName} - keeping first");
                continue;
            }

            seen.Add(entry.Id);
            result.Add(entry);
        }

        return result;
    }

    private static PackageEntry? ParsePackageEntry(LuaValue value, string sourceName)
    {
        if (value.Type == LuaValueType.String)
        {
            return new PackageEntry(value.Read<string>(), null);
        }
        else if (value.Type == LuaValueType.Table)
        {
            var table = value.Read<LuaTable>();
            var idVal = table["id"];
            var versionVal = table["version"];

            if (idVal.Type != LuaValueType.String)
            {
                throw new InvalidOperationException($"Invalid package entry in {sourceName}: missing 'id'");
            }

            return new PackageEntry(
                idVal.Read<string>(),
                versionVal.Type == LuaValueType.String ? versionVal.Read<string>() : null
            );
        }

        throw new InvalidOperationException($"Invalid package entry type in {sourceName}");
    }
}