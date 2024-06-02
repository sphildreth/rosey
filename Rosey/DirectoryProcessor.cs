using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;

namespace Rosey;

public class DirectoryProcessor(string[] directoryRegexToDelete, string[] fileRegexsToDelete)
{
    private readonly string[] _directoryRegexToDelete = directoryRegexToDelete;
    private readonly string[] _fileRegexsToDelete = fileRegexsToDelete;

    public bool DoDeleteDirectory(DirectoryInfo dir) => _directoryRegexToDelete.Length != 0 && _directoryRegexToDelete.Any(dirRegexToDelete => Regex.IsMatch(dir.Name, dirRegexToDelete, RegexOptions.IgnoreCase));
    
    public bool DoDeleteFile(FileInfo file) => _fileRegexsToDelete.Length != 0 && _fileRegexsToDelete.Any(fileRegexToDelete => Regex.IsMatch(file.Name, fileRegexToDelete, RegexOptions.IgnoreCase));

    
    public bool Process(DirectoryInfo directoryToProcess, bool readOnly)
    {
        if (!directoryToProcess.Exists)
        {
            return false;
        }

        foreach (var dir in directoryToProcess.EnumerateDirectories("*", SearchOption.AllDirectories))
        {
            if (!DoDeleteDirectory(dir))
            {
                continue;
            }
            Console.WriteLine($": - Deleting Directory [{dir.Name}]");
            dir.Delete(true);
        }
        foreach (var file in directoryToProcess.EnumerateFiles("*.*", SearchOption.AllDirectories))
        {
            if (!DoDeleteFile(file))
            {
                continue;
            }
            Console.WriteLine($": - Deleting File [{file.Name}]");
            file.Delete();
        }
        DeleteEmptyDirs(directoryToProcess.FullName);
        return true;
    }
    
    static void DeleteEmptyDirs(string dir)
    {
        if (string.IsNullOrEmpty(dir))
        {
            throw new ArgumentException(
                "Starting directory is a null reference or an empty string",
                nameof(dir));
        }
        try
        {
            foreach (var d in Directory.EnumerateDirectories(dir))
            {
                DeleteEmptyDirs(d);
            }

            var entries = Directory.EnumerateFileSystemEntries(dir);

            if (entries.Any())
            {
                return;
            }
            try
            {
                Console.WriteLine($": - Deleting Empty Directory [{dir}]");
                Directory.Delete(dir);
            }
            catch (UnauthorizedAccessException) { }
            catch (DirectoryNotFoundException) { }
        }
        catch (UnauthorizedAccessException) { }
    }    
}