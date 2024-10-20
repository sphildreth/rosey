using System;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using Xabe.FFmpeg;

namespace Rosey.Models
{
    public sealed class FileMetaInfo
    {

        private const int MinimumValidYear = 1895;

        private IVideoStream _videoStream;

        public FileInfo FileInfo { get; }

        public int? Rotation => _videoStream?.Rotation;

        public int? Forced => _videoStream?.Forced;

        public int? Default => _videoStream?.Default;

        public long? Bitrate => _videoStream?.Bitrate;

        public string Ratio => _videoStream?.Ratio;

        public double? Framerate => _videoStream?.Framerate;

        public int? Height => _videoStream?.Height;

        public int? Width => _videoStream?.Width;

        public TimeSpan? Duration => _videoStream?.Duration;

        public string PixelFormat => _videoStream?.PixelFormat;

        public string Title => TitleFromFileName(FileInfo.FullName);

        public FileMetaInfo(FileInfo fileInfo)
        {
            FileInfo = fileInfo;
        }

        public async Task<bool> LoadInfo()
        {
            try
            {
                var info = await FFmpeg.GetMediaInfo(FileInfo.FullName).ConfigureAwait(false);
                _videoStream = info?.VideoStreams?.First();                 
                return true;
            }
            catch (Exception ex)
            {
                Trace.WriteLine($"Error Reading Information From File [{ FileInfo.FullName }] Ex [{ ex }]");
            }
            return false;
        }

        public static string TitleFromFileName(string fullName)
        {
            var fileInfo = new FileInfo(fullName);
            string[] parts = new string[0];
            fullName = Regex.Replace(fullName, @"www[^\s]+", "").Replace('.', ' ').Replace('_', ' ').Replace('-', ' ');
            if (fullName.IndexOf(' ') > 0)
            {
                parts = fullName.Split(' ');
            }
            if (parts.Length == 0)
            {
                return CultureInfo.CurrentCulture.TextInfo.ToTitleCase(fullName);
            }
            StringBuilder result = new StringBuilder(CultureInfo.CurrentCulture.TextInfo.ToTitleCase(parts[0].Trim()));
            foreach (var part in parts.Skip(1))
            {
                if (string.IsNullOrWhiteSpace(part))
                {
                    continue;
                }
                if (fileInfo.Extension.ToLower().EndsWith(part))
                {
                    break;
                }
                if (int.TryParse(
                        part.Replace('[', ' ').Replace('{', ' ').Replace('(', ' ').Replace(')', ' ')
                            .Replace('}', ' ').Replace(']', ' ').Trim(), out int year))
                {
                    if (year > MinimumValidYear)
                    {
                        result.Append(" (").Append(year).Append(")");
                        break;
                    }
                }

                result.Append(' ').Append(CultureInfo.CurrentCulture.TextInfo.ToTitleCase(part));
            }
            return result.ToString().Trim();
        }
    }
}