using System.Collections.Generic;

namespace NTIX.Core;

internal static class ConsoleHelper
{
    private static readonly Dictionary<string, ConsoleColor> SourceColors = new(StringComparer.OrdinalIgnoreCase)
    {
        ["winget"]     = ConsoleColor.DarkMagenta,
        ["chocolatey"] = ConsoleColor.Blue,
        ["scoop"]      = ConsoleColor.Magenta,
    };

    public static void WriteWarning(string message)
    {
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.Error.WriteLine($"[warn] {message}");
        Console.ResetColor();
    }

    public static void WriteError(string message)
    {
        Console.ForegroundColor = ConsoleColor.Red;
        Console.Error.WriteLine($"[error] {message}");
        Console.ResetColor();
    }

    public static void WritePackageLine(string source, string id, string version)
    {
        Console.Write("  ");
        if (SourceColors.TryGetValue(source, out var color))
            Console.ForegroundColor = color;
        Console.Write(source);
        Console.ResetColor();
        Console.WriteLine($": {id} ({version})");
    }

    public static void WriteSectionHeader(string text, ConsoleColor color)
    {
        Console.ForegroundColor = color;
        Console.WriteLine(text);
        Console.ResetColor();
    }
}
