using MoonSharp.Interpreter;
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
        UserData.RegisterAssembly();

        var lua = new Script();
        
        var directory = Path.GetDirectoryName(configPath) ?? "";
        if (!string.IsNullOrEmpty(directory))
        {
            var packageTable = lua.Globals.Get("package").Table;
            var currentPath = packageTable.Get("path").String;
            packageTable.Set("path", DynValue.NewString(currentPath + ";" + directory.Replace('\\', '/') + "/?.lua"));
        }

        try
        {
            var result = lua.DoString(luaScript);
            return ParseConfig(result, configPath);
        }
        catch (SyntaxErrorException ex)
        {
            throw new InvalidOperationException($"Lua syntax error: {ex.Message}");
        }
        catch (ScriptRuntimeException ex)
        {
            throw new InvalidOperationException($"Lua runtime error: {ex.Message}");
        }
    }

    private static NTIXConfig ParseConfig(DynValue result, string configPath)
    {
        if (result.Type != DataType.Table)
            throw new InvalidOperationException("Config must return a table");

        var table = result.Table;

        var optionsTable = table.Get("options").Table;
        if (optionsTable == null)
            throw new InvalidOperationException("Config error: missing top-level 'options' table");

        var wingetOptions = ParseWingetOptions(optionsTable);
        var chocoOptions = ParseChocoOptions(optionsTable);
        var scoopOptions = ParseScoopOptions(optionsTable);

        var options = new NTIXOptions(wingetOptions, chocoOptions, scoopOptions);

        var pkgsTable = table.Get("pkgs").Table;
        if (pkgsTable == null)
            throw new InvalidOperationException("Config error: missing top-level 'pkgs' table");

        var wingetPackages = ExtractPackages(pkgsTable.Get("winget"), "pkgs.winget");
        var chocoPackages = ExtractPackages(pkgsTable.Get("chocolatey"), "pkgs.chocolatey");
        var scoopPackages = ExtractPackages(pkgsTable.Get("scoop"), "pkgs.scoop");

        return new NTIXConfig(options, wingetPackages, chocoPackages, scoopPackages);
    }

    private static WingetOptions ParseWingetOptions(Table optionsTable)
    {
        var wingetTable = optionsTable.Get("winget").Table;
        if (wingetTable == null)
            return new WingetOptions();

        return new WingetOptions(
            Enable: wingetTable.Get("enable").Boolean,
            AcceptAgreements: wingetTable.Get("acceptAgreements").Boolean,
            Interactive: wingetTable.Get("interactive").Boolean
        );
    }

    private static ChocoOptions ParseChocoOptions(Table optionsTable)
    {
        var chocoTable = optionsTable.Get("chocolatey").Table;
        if (chocoTable == null)
            return new ChocoOptions();

        return new ChocoOptions(
            Enable: chocoTable.Get("enable").Boolean,
            Yes: chocoTable.Get("yes").Boolean
        );
    }

    private static ScoopOptions ParseScoopOptions(Table optionsTable)
    {
        var scoopTable = optionsTable.Get("scoop").Table;
        if (scoopTable == null)
            return new ScoopOptions();

        var buckets = new List<string> { "main", "extras", "versions" };
        var bucketsVal = scoopTable.Get("buckets");
        if (bucketsVal.Type == DataType.Table)
        {
            buckets.Clear();
            foreach (var kvp in bucketsVal.Table.Pairs)
            {
                if (kvp.Value.Type == DataType.String)
                    buckets.Add(kvp.Value.String);
            }
        }

        return new ScoopOptions(
            Enable: scoopTable.Get("enable").Boolean,
            Buckets: buckets
        );
    }

    private static List<PackageEntry> ExtractPackages(DynValue pkgsVal, string sourceName)
    {
        var result = new List<PackageEntry>();
        
        if (pkgsVal == null || pkgsVal.Type != DataType.Table)
            return result;

        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        
        foreach (var kvp in pkgsVal.Table.Pairs)
        {
            if (kvp.Key.Type != DataType.Number)
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

    private static PackageEntry? ParsePackageEntry(DynValue value, string sourceName)
    {
        if (value.Type == DataType.String)
        {
            return new PackageEntry(value.String, null);
        }
        else if (value.Type == DataType.Table)
        {
            var table = value.Table;
            var idVal = table.Get("id");
            var versionVal = table.Get("version");

            if (idVal.Type != DataType.String)
            {
                throw new InvalidOperationException($"Invalid package entry in {sourceName}: missing 'id'");
            }

            return new PackageEntry(
                idVal.String,
                versionVal.Type == DataType.String ? versionVal.String : null
            );
        }

        throw new InvalidOperationException($"Invalid package entry type in {sourceName}");
    }
}