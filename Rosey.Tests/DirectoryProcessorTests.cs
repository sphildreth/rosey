using System.IO;
using Xunit;

namespace Rosey.Tests;

public class DirectoryProcessorTests
{
    [Fact]
    public void ValidateDoDeleteFiles()
    {
        var fileProcessor = new DirectoryProcessor([],new[] { "\\w+.txt" },new[] { "\\w+.mp3"});
        Assert.False(fileProcessor.DoDeleteFile(new FileInfo("Batman.jpg")));
        Assert.True(fileProcessor.DoDeleteFile(new FileInfo("Batman.txt")));
    }
    
    [Fact]
    public void ValidateNeverDeleteFiles()
    {
        var fileProcessor = new DirectoryProcessor([],new[] { "\\w+.txt" }, new[] { "\\w+.mp3"});
        Assert.False(fileProcessor.DoDeleteFile(new FileInfo("Batman.jpg")));
        Assert.False(fileProcessor.DoDeleteFile(new FileInfo("Batman.mp3")));
        Assert.False(fileProcessor.DoDeleteFile(new FileInfo("Batman.flac")));
        Assert.True(fileProcessor.DoDeleteFile(new FileInfo("Batman.txt")));
    }    
    
    [Fact]
    public void ValidateDoDeleteDirectories()
    {
        var fileProcessor = new DirectoryProcessor(new[] { "(Subs)" }, [], []);
        Assert.False(fileProcessor.DoDeleteDirectory(new DirectoryInfo("Batman")));
        Assert.True(fileProcessor.DoDeleteDirectory(new DirectoryInfo("Subs")));
    }    
}