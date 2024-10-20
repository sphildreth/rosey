using Rosey.Models;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using Rosey.Extensions;

namespace Rosey
{
    public sealed class FileProcessor
    {
        public bool Process(DirectoryInfo toDirectory, FileInfo file, bool readOnly)
        {
            // Get the new title for the full filename
            var fileTitle = FileMetaInfo.TitleFromFileName(file.Name);

            // See if target folder exists by title name, if not create it
            var targetDir = new DirectoryInfo(Path.Combine(toDirectory.FullName, fileTitle.ToFileNameFriendly()));
            if (!targetDir.Exists)
            {
                if (!readOnly)
                {
                    targetDir.Create();
                }
                Trace.WriteLine($": + Created Directory [{ targetDir }]");
            }

            // Get all other files matching the filename
            var filesToMove = new List<FileInfo>();
            foreach (var f in Directory.EnumerateFiles(file.Directory!.FullName, $"{ file.Name.Replace(file.Extension, "") }*.*"))
            {
                filesToMove.Add(new FileInfo(f));
            }

            string currentFileBeingProcessed = null;
            try
            {
                // For each file to move rename to new title + existing extension in target folder
                foreach (var f in filesToMove)
                {
                    currentFileBeingProcessed = f.FullName;
                    var newName = Path.Combine(targetDir.FullName, $"{ fileTitle }{ f.Extension }".ToFileNameFriendly());
                    if (!readOnly)
                    {
                        f.MoveTo(newName, true);
                    }
                    Trace.WriteLine($": > Moved File [{ f.FullName }] -> [{ newName }]");
                }
            }
            catch (System.Exception ex)
            {
                Trace.WriteLine($"!! Error Processing File [{ currentFileBeingProcessed }] Ex [{ ex }]");
            }

            return true;
        }
    }
}