# pre deploy
- [x] building an index
- [x] <base> tags + generating routes on serve
- [x] menu bar

- [x] right now, the blog index page is static. **this needs to be on the fly**
- [x] server supports webm locally
- [x] footnote formatting
- [x] conditional includes
    - [x] conditional syntax highlighting inclusion if page contains code
    - [x] conditional math font inclusion if page contains math (need to choose a math font)

- [ ] routing table builds gets created on site build
- [ ] footer with deployment information (commit), last modified, etc
    - [x] commit (no link)

- [x] convert images to webp, wrap them in links to png or jpg
    - [x] wrap in link
    - [x] link to jpg version
    - [x] use source tags
    - [ ] maybe they should be resized to the viewport width: we can do this with while let in render step
    - [ ] images definitely need a size... how to do this? To prevent content rearranging
- [ ] don't build drafts
- [ ] comment engine 
    - [ ] rebuild the web server to support this
- [ ] incremental rebuilds with file hashing
- [ ] adopt old posts to new format
    - [x] fh
    - [ ] scraper thing
- [ ] table of contents (sidecar?)
    can do this with while let

# post deploy
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
