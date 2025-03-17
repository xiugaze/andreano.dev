# Before Done


**NOTE TO SELF**
Table of contents: works right now, but only because
of {{{ toc }}} in base template html, not in the markdown thing. 
Maybe we need a pre-render step for inserting the TOC html
at the correct location in the markdown? Or inserting the rendered
html body content into memory and then re-rendering for page...
Also, header text is consumed with parser.next(), so doesn't
end up in final page. Also, headers with the ' character get cut 
for some reason, why is that? this should probably get fixed. 

Also, because of relative paths, if the href of an a is like #header-1, the 
browser requests post/#header-1 instead of post#header-1. How to fix this?


## colors

- [x] fix color fading from default on homepage
- [x] cube color
- [ ] favicon colors
- [ ] fenced code colors
- [ ] about page list colors


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
    - [x] commit (no link)

- [x] convert images to webp, wrap them in links to png or jpg
    - [x] wrap in link
    - [x] link to jpg version
    - [x] use source tags
    - [ ] maybe they should be resized to the viewport width: we can do this with while let in render step
    - [x] images definitely need a size... how to do this? To prevent content rearranging
- [ ] don't build drafts
- [ ] incremental rebuilds with file hashing
- [ ] adopt old posts to new format
    - [x] fh
    - [ ] scraper thing
- [x] table of contents (sidecar?)
    can do this with while let

## chrome problems
- [ ] math fonts
- [ ] theme switch transitions
    - [x] calling this fixed 

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
- [ ] md directory can have scripts and css, should get copied and included automatically
