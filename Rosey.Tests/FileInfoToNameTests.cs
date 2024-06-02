using Rosey.Models;
using System;
using Xunit;

namespace Rosey.Tests
{
    public class FileInfoToNameTests
    {
        [Theory] // 
        [InlineData("1917.2019.1080p.WEBRip.x264.AAC5.1-[YTS.MX].mp4", "1917 (2019)")]
        [InlineData("5.Flights.Up.2014.HDRip.XviD.AC3-EVO.avi", "5 Flights Up (2014)")]
        [InlineData("Alexander.and.the.Terrible,.Horrible,.No.Good,.Very.Bad.Day.2014.1080p.BluRay.x264.YIFY.mp4", "Alexander And The Terrible, Horrible, No Good, Very Bad Day (2014)")]
        [InlineData("Anchorman[Unrated]DVDRip.Xvid.2004-tots.avi", "Anchorman[Unrated]Dvdrip Xvid (2004)")]
        [InlineData("Annie_[1982].mkv", "Annie (1982)")]
        [InlineData("Dawn of the Dead.avi", "Dawn Of The Dead")]
        [InlineData("Elf.2003.720p.BrRip.x264.YIFY.mp4", "Elf (2003)")]
        [InlineData("goingDistance-faye-xvid.avi", "Goingdistance Faye Xvid")]
        [InlineData("Home.2015.720p.BluRay.x264.YIFY.mp4", "Home (2015)")]
        [InlineData("here.comes.the.boom.dvdrip.xvid-itch.avi", "Here Comes The Boom Dvdrip Xvid Itch")]
        [InlineData("Joker BDx265.www.pctnew.org.mkv", "Joker Bdx265")]
        [InlineData("The Toy (1982) 1080p By M2E.mkv", "The Toy (1982)")]
        [InlineData("Lord Of The Rings The Fellowship of the Ring [2001].mp4", "Lord Of The Rings The Fellowship Of The Ring (2001)")]
        [InlineData("My_Life_In_Ruins_[2009].avi", "My Life In Ruins (2009)")]
        [InlineData("truth_About_Cats_and_Dogs_(1996).avi", "Truth About Cats And Dogs (1996)")]
        [InlineData("Trolls 2016 1080p BluRay x264 DTS-JYK.mkv", "Trolls (2016)")]
        [InlineData("Wreck.it.Ralph.2012.1080p.BrRip.x264.BOKUTOX.YIFY.mp4", "Wreck It Ralph (2012)")] 
        [InlineData("XMEN.Origins.2009.XviD.avi", "XMEN Origins (2009)")]
        [InlineData("Men_In_Black_3_2015", "Men In Black 3 (2015)")]
        [InlineData("Terminator.2.1991.1080p.BluRay.x264-[YTS.AG]", "Terminator 2 (1991)")]
        [InlineData("Mr. Mom (1983) [1080p] [YTS.AG].mkv", "Mr Mom (1983)")]
        public void TitleFromFileName(string fileName, string shouldBe) => Assert.Equal(shouldBe, FileMetaInfo.TitleFromFileName(fileName));
    }
}
