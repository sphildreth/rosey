using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using Microsoft.Extensions.Configuration;

namespace Rosey
{
    class Program
    {
        static void Main(params string[] args)
        {
            var config = new ConfigurationBuilder()
                .SetBasePath(AppDomain.CurrentDomain.BaseDirectory)
                .AddJsonFile("appsettings.json", optional: false)
                .AddCommandLine(args)
                .Build();          
            
            // Define a trace listener to direct trace output from this method
            // to the console.
            ConsoleTraceListener consoleTracer;

            // Check the command line arguments to determine which
            // console stream should be used for trace output.
            if ((args.Length > 0) && (args[0].Equals("/stderr", StringComparison.OrdinalIgnoreCase)))
            // Initialize the console trace listener to write
            // trace output to the standard error stream.
            {
                consoleTracer = new ConsoleTraceListener(true);
            }
            else
            {
                // Initialize the console trace listener to write
                // trace output to the standard output stream.
                consoleTracer = new ConsoleTraceListener();
            }
            // Set the name of the trace listener, which helps identify this
            // particular instance within the trace listener collection.
            consoleTracer.Name = "mainConsoleTracer";

            // Write the initial trace message to the console trace listener.
            consoleTracer.WriteLine(DateTime.Now.ToString(CultureInfo.InvariantCulture) + " [" + consoleTracer.Name + "] - Starting output to trace listener.");

            // Add the new console trace listener to
            // the collection of trace listeners.
            Trace.Listeners.Add(consoleTracer);

            if (args.Length == 0)
            {
                Console.WriteLine("Invalid Directory");
                return;
            }
            var directory = args.First();
            if(string.IsNullOrWhiteSpace(directory))
            {
                Console.WriteLine("Invalid Directory");
                return;
            }
            var dir = new DirectoryInfo(directory);
            if(!dir.Exists)
            {
                Console.WriteLine($"Invalid Directory [{ directory }]");
                return;
            }
            var toDirectory = args.Length > 1 ? args.Skip(1).First() : directory;
            var toDir = new DirectoryInfo(toDirectory);
            if (!toDir.Exists)
            {
                Console.WriteLine($"Invalid To Directory [{ toDirectory }]");
                return;
            }
            var fileProcessor = new FileProcessor();
            foreach (var df in dir.GetDirectories("*.*", SearchOption.TopDirectoryOnly))
            {
                Console.WriteLine($": Processing Directory [{ df.FullName }]");
                
                foreach (var f in df.GetFiles())
                {
                    if (FilesToProcess.Contains(f.Extension.TrimStart('.').ToLower()))
                    {
                        if (!fileProcessor.Process(toDir, f, false))
                        {
                            Console.WriteLine($"Unable to Process [{f.FullName}]");
                            return;
                        }
                    }
                }
            }
            var folderProcessor = new DirectoryProcessor(
                config.GetSection("Rosey:DirectoriesToDelete").Get<string[]>(), 
                config.GetSection("Rosey:FilesToDelete").Get<string[]>());
            folderProcessor.Process(dir, false);
        }

        private static IEnumerable<string> FilesToProcess => new List<string> { "avi", "avichd", "flv", "mov", "mp4", "mkv", "webm" };

    }
}
