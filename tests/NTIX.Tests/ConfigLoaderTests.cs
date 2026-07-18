using FluentAssertions;
using NTIX.Core.Config;

namespace NTIX.Tests;

public class ConfigLoaderTests
{
    [Fact]
    public void LoadFromString_ValidConfig_ReturnsNTIXConfig()
    {
        var lua = """
            options = {
                winget = { enable = true, acceptAgreements = true, interactive = false },
                chocolatey = { enable = true, yes = true },
                scoop = { enable = true, buckets = { "main", "extras" } }
            }
            pkgs = {
                winget = { "Microsoft.VisualStudioCode" },
                chocolatey = { "git" },
                scoop = { "fd" }
            }
            return { options = options, pkgs = pkgs }
            """;

        var config = ConfigLoader.LoadFromString(lua, "test.lua");
        config.Should().NotBeNull();
        config.Options.Winget.Enable.Should().BeTrue();
        config.Options.Chocolatey.Enable.Should().BeTrue();
        config.Options.Scoop.Enable.Should().BeTrue();
        config.WingetPackages.Should().HaveCount(1);
        config.ChocoPackages.Should().HaveCount(1);
        config.ScoopPackages.Should().HaveCount(1);
    }

    [Fact]
    public void LoadFromString_MissingInteractiveField_DefaultsToFalse()
    {
        var lua = """
            options = {
                winget = { enable = true, acceptAgreements = true },
                chocolatey = { enable = true, yes = true },
                scoop = { enable = true, buckets = { "main", "extras" } }
            }
            pkgs = {
                winget = { "Microsoft.VisualStudioCode" },
                chocolatey = { "git" },
                scoop = { "fd" }
            }
            return { options = options, pkgs = pkgs }
            """;

        var config = ConfigLoader.LoadFromString(lua, "test.lua");
        config.Should().NotBeNull();
        config.Options.Winget.Enable.Should().BeTrue();
        config.Options.Winget.Interactive.Should().BeFalse();
    }

    [Fact]
    public void Load_MissingConfigFile_ThrowsFileNotFoundException()
    {
        var act = () => ConfigLoader.Load("nonexistent.lua");
        act.Should().Throw<FileNotFoundException>()
            .WithMessage("Config file not found: nonexistent.lua");
    }

    [Fact]
    public void LoadFromString_MissingOptionsTable_ThrowsInvalidOperationException()
    {
        var lua = """
            pkgs = {
                winget = { "test" }
            }
            return { pkgs = pkgs }
            """;

        var act = () => ConfigLoader.LoadFromString(lua, "test.lua");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("Config error: missing top-level 'options' table");
    }

    [Fact]
    public void LoadFromString_MissingPkgsTable_ThrowsInvalidOperationException()
    {
        var lua = """
            options = {
                winget = { enable = true }
            }
            return { options = options }
            """;

        var act = () => ConfigLoader.LoadFromString(lua, "test.lua");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("Config error: missing top-level 'pkgs' table");
    }

    [Fact]
    public void LoadFromString_MissingWingetInOptions_DefaultsToEmpty()
    {
        var lua = """
            options = {
                chocolatey = { enable = true }
            }
            pkgs = {
                winget = { "test" }
            }
            return { options = options, pkgs = pkgs }
            """;

        var config = ConfigLoader.LoadFromString(lua, "test.lua");
        config.Should().NotBeNull();
        config.Options.Winget.Enable.Should().BeFalse();
    }

    [Fact]
    public void LoadFromString_WithImport_SingleFile_MergesOptionsAndPackages()
    {
        var testDir = Path.Combine(Path.GetTempPath(), "ntix_test_imports", Guid.NewGuid().ToString());
        Directory.CreateDirectory(testDir);

        try
        {
            var baseConfig = """
                return {
                    options = {
                        winget = { enable = true, acceptAgreements = true },
                        chocolatey = { enable = true, yes = true }
                    },
                    pkgs = {
                        winget = { "Microsoft.VisualStudioCode", "Git.Git" },
                        chocolatey = { "git" }
                    }
                }
                """;
            File.WriteAllText(Path.Combine(testDir, "base.lua"), baseConfig);

            var mainConfig = """
                import("base.lua")
                return { options = options, pkgs = pkgs }
                """;

            var config = ConfigLoader.LoadFromString(mainConfig, Path.Combine(testDir, "main.lua"));

            config.Should().NotBeNull();

            config.Options.Winget.Enable.Should().BeTrue();
            config.Options.Winget.AcceptAgreements.Should().BeTrue();
            config.Options.Chocolatey.Enable.Should().BeTrue();
            config.Options.Chocolatey.Yes.Should().BeTrue();

            config.WingetPackages.Should().HaveCount(2);
            config.WingetPackages.Select(p => p.Id).Should().Contain("Microsoft.VisualStudioCode");
            config.WingetPackages.Select(p => p.Id).Should().Contain("Git.Git");

            config.ChocoPackages.Should().HaveCount(1);
            config.ChocoPackages[0].Id.Should().Be("git");
        }
        finally
        {
            Directory.Delete(testDir, true);
        }
    }

    [Fact]
    public void LoadFromString_WithImport_MissingFile_ThrowsFileNotFoundException()
    {
        var lua = """
            import("nonexistent.lua")
            return { options = options, pkgs = pkgs }
            """;

        var act = () => ConfigLoader.LoadFromString(lua, "test.lua");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*nonexistent.lua*");
    }

    [Fact]
    public void LoadFromString_WithImport_NestedImports_MergeCorrectly()
    {
        var testDir = Path.Combine(Path.GetTempPath(), "ntix_test_imports", Guid.NewGuid().ToString());
        var nestedDir = Path.Combine(testDir, "nested");
        Directory.CreateDirectory(nestedDir);

        try
        {
            var baseConfig = """
                return {
                    options = { winget = { enable = true } },
                    pkgs = { winget = { "Base.Package" } }
                }
                """;
            File.WriteAllText(Path.Combine(nestedDir, "base.lua"), baseConfig);

            var extConfig = """
                import("base.lua")
                return {
                    options = options,
                    pkgs = { winget = { "Extended.Package" } }
                }
                """;
            File.WriteAllText(Path.Combine(nestedDir, "ext.lua"), extConfig);

            var mainConfig = """
                import("nested/ext.lua")
                return { options = options, pkgs = pkgs }
                """;

            var config = ConfigLoader.LoadFromString(mainConfig, Path.Combine(testDir, "main.lua"));

            config.Should().NotBeNull();
            config.Options.Winget.Enable.Should().BeTrue();

            config.WingetPackages.Should().HaveCount(2);
            config.WingetPackages.Select(p => p.Id).Should().Contain("Base.Package");
            config.WingetPackages.Select(p => p.Id).Should().Contain("Extended.Package");
        }
        finally
        {
            Directory.Delete(testDir, true);
        }
    }

    [Fact]
    public void LoadFromString_WithImport_PackageDeduplication_ById()
    {
        var testDir = Path.Combine(Path.GetTempPath(), "ntix_test_imports", Guid.NewGuid().ToString());
        Directory.CreateDirectory(testDir);

        try
        {
            var pkg1Config = """
                return {
                    options = { winget = { enable = true } },
                    pkgs = {
                        winget = { 
                            "Unique.Package",
                            { id = "Duplicate.Package", version = "1.0" }
                        }
                    }
                }
                """;
            File.WriteAllText(Path.Combine(testDir, "pkg1.lua"), pkg1Config);

            var pkg2Config = """
                return {
                    pkgs = {
                        winget = {
                            { id = "Duplicate.Package", version = "2.0" },
                            "Another.Unique"
                        }
                    }
                }
                """;
            File.WriteAllText(Path.Combine(testDir, "pkg2.lua"), pkg2Config);

            var mainConfig = """
                import({ "pkg1.lua", "pkg2.lua" })
                return { options = options, pkgs = pkgs }
                """;

            var config = ConfigLoader.LoadFromString(mainConfig, Path.Combine(testDir, "main.lua"));

            config.Should().NotBeNull();
            config.WingetPackages.Should().HaveCount(3);
            config.WingetPackages.Select(p => p.Id).Should().Contain("Unique.Package");
            config.WingetPackages.Select(p => p.Id).Should().Contain("Another.Unique");

            var dup = config.WingetPackages.First(p => p.Id == "Duplicate.Package");
            dup.Version.Should().Be("2.0");
        }
        finally
        {
            Directory.Delete(testDir, true);
        }
    }

    [Fact]
    public void LoadFromString_WithImport_ArrayOfPaths_ImportsAll()
    {
        var testDir = Path.Combine(Path.GetTempPath(), "ntix_test_imports", Guid.NewGuid().ToString());
        Directory.CreateDirectory(testDir);

        try
        {
            var wingetConfig = """
                return {
                    pkgs = { winget = { "Winget.Package" } }
                }
                """;
            File.WriteAllText(Path.Combine(testDir, "winget.lua"), wingetConfig);

            var scoopConfig = """
                return {
                    pkgs = { scoop = { "Scoop.Package" } }
                }
                """;
            File.WriteAllText(Path.Combine(testDir, "scoop.lua"), scoopConfig);

            var mainConfig = """
                import({ "winget.lua", "scoop.lua" })
                return { options = options, pkgs = pkgs }
                """;

            var config = ConfigLoader.LoadFromString(mainConfig, Path.Combine(testDir, "main.lua"));

            config.Should().NotBeNull();
            config.WingetPackages.Should().HaveCount(1);
            config.WingetPackages[0].Id.Should().Be("Winget.Package");

            config.ScoopPackages.Should().HaveCount(1);
            config.ScoopPackages[0].Id.Should().Be("Scoop.Package");
        }
        finally
        {
            Directory.Delete(testDir, true);
        }
    }

    [Fact]
    public void LoadFromString_WithImport_DeepMergeOptions_PreservesNestedKeys()
    {
        var testDir = Path.Combine(Path.GetTempPath(), "ntix_test_imports", Guid.NewGuid().ToString());
        Directory.CreateDirectory(testDir);

        try
        {
            var baseConfig = """
                return {
                    options = {
                        winget = { 
                            enable = true, 
                            acceptAgreements = true,
                            interactive = false
                        },
                        scoop = { 
                            enable = true,
                            buckets = { "main" }
                        }
                    },
                    pkgs = { winget = {} }
                }
                """;
            File.WriteAllText(Path.Combine(testDir, "base.lua"), baseConfig);

            var mainConfig = """
                import("base.lua")
                -- Modify global options table
                options.winget.interactive = true
                options.scoop.buckets = { "main", "extras" }
                return { options = options, pkgs = pkgs }
                """;

            var config = ConfigLoader.LoadFromString(mainConfig, Path.Combine(testDir, "main.lua"));

            config.Should().NotBeNull();
            config.Options.Winget.Enable.Should().BeTrue();
            config.Options.Winget.AcceptAgreements.Should().BeTrue();
            config.Options.Winget.Interactive.Should().BeTrue();

            config.Options.Scoop.Enable.Should().BeTrue();
            config.Options.Scoop.Buckets.Should().Contain("main");
            config.Options.Scoop.Buckets.Should().Contain("extras");
        }
        finally
        {
            Directory.Delete(testDir, true);
        }
    }
}