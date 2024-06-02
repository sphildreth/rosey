using System.Text.RegularExpressions;

namespace Rosey.Extensions;

public static class StringExtentions
{

    public static string ToFolderNameFriendly(this string input)
    {
        if (string.IsNullOrEmpty(input))
        {
            return null;
        }
        input = input.Replace("$", "s");
        return Regex.Replace(Sanitizer.SanitizeFilename(input, ' '), @"\s+", " ").Trim().TrimEnd('.');
    }
    
    public static string ToFileNameFriendly(this string input)
    {
        if (string.IsNullOrEmpty(input))
        {
            return null;
        }

        return Regex.Replace(Sanitizer.SanitizeFilename(input, ' '), @"\s+", " ").Trim();
    }
}