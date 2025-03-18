# Before Done

**NOTE TO SELF**
Headers with the ' character get cut 
for some reason, why is that? this should probably get fixed. 

**NOTE TO SELF**
Fix spacing on frontpage

## nix stuff
- [ ] build ssg
- [ ] build site (run ssg)
- [ ] deploy site (run server)


## colors

- [x] fix color fading from default on homepage
- [x] cube color
- [ ] favicon colors
- [x] fenced code colors
    - done for now, but should figure out how to do custom themes
- [x] about page list colors


## server
- [x] routing table builds gets created on site build
- [x] server supports webm locally
- [ ] routing table includes index pages
- [ ] comment engine 
    - [ ] rebuild the web server to support this

## page generation
- [x] building an index
- [x] <base> tags + generating routes on serve
- [x] menu bar
- [x] footnote formatting
- [x] conditional includes
    - [x] conditional syntax highlighting inclusion if page contains code
    - [x] conditional math font inclusion if page contains math (need to choose a math font)

- [x] right now, the blog index page is static. **this needs to be on the fly**
- [ ] footer with deployment information (commit), last modified, etc
    - [x] commit

- [x] convert images to webp, wrap them in links to png or jpg
    - [x] wrap in link
    - [x] link to jpg version
    - [x] use source tags
    - [ ] maybe the actual image should be resized to the viewport width: we can do this with while let in render step
    - [x] images definitely need a size... how to do this? To prevent content rearranging
- [x] don't build drafts
- [ ] incremental rebuilds with file hashing
- [ ] adopt old posts to new format
    - [x] fh
    - [x] scraper thing
- [x] table of contents (sidecar?)
    can do this with while let
- [x] index page mobile formatting

## chrome problems
- [ ] math fonts
- [x] theme switch transitions
    - [x] calling this fixed, but it's not really. Nothing I can do here.

# After Done
- [ ] collapsable headings
- [x] menu bar as component
- [x] include absolute styles from frontmatter
- [ ] include absolute scripts from frontmatter
- [ ] include relative scripts from directory
- [ ] include relative scripts from directory

- [ ] for latex2mathml, image, consume closing tag
- [ ] rss feed
- [ ] text-only sitemap generation
- [ ] index page for subfolders in general? 
- [ ] md directory can have scripts and css, should get copied and included automatically
