namespace NTIX.Core.Models;

public record ImportNode(string Path, List<ImportNode> Children)
{
    public ImportNode(string path) : this(path, new()) { }
}
