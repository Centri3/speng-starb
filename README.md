# speng-starb (Star Browser Utilities)

A program to provide QOL patches for the Star browser in [SpaceEngine](https://spaceengine.org/). This is unfinished, so make sure to [suggest new patches*](https://github.com/Centri3/speng-starb/issues)! (or create [pull requests](https://github.com/Centri3/speng-starb/pulls) to fix my spaghetti code!)

***Which can be realistically added!**

## Current Patches

* No Max Search Radius
  * Remove the 100pc (326.16ly) search radius limit, or set your own!
  * WARNING: Setting this too high while searching for rarer stars (Neutron stars, Black holes, etc) will lag the game, and possibly crash it.
* No Search Locking
  * Tries to fix SE's search button locking occasionally on newer versions.
  * NOTE: This doesn't entirely patch it, but uses a much better method which allows\nyou to press stop and clear to fix it, rather than needing to input StarBrowserReset in the console.
* Double Click Start Search
  * Soon™️ (:
* Accurate Temperature Filter
  * The Star browser currently uses Current Temperature at time January 1st 2000, 12:00:00, this forces it to use Average Temperature instead.
  * NOTE: This can be even less accurate at times than current temperature, but isusually much closer to what your filters are.
* ESI Filter
  * Adds the long requested ESI Filter. That's it.

## Installation

Simply extract the archive in releases. Unfortunately, this is going to be Windows only possibly forever.

## Anti-virus Notice

If your anti-virus goes off on this, that's good! The method this uses to mod SE is code injection: this can be used to create malware, but can also be used to create mods/patches for games, which is what this does. Just add an exception for this.

If you don't believe me, feel free to browse and build (or even modify) the source yourself.

## Spaghetti Notice

Your eyes don't decieve you; this code sucks. The majority of this is rushed, (though that hasn't reduced the quality of the final product, I still made sure nothing broke) and if you have the expertise to fix it, please do!

## To SE Devs

If you read this, can you please lecture me on how SE's GUI system works?
