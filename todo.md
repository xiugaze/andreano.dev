# Before Done

[**STL Viewer**](https://github.com/ottobonn/stl-viewer)
- this talks about how to actually use it: https://tonybox.net/posts/simple-stl-viewer/

## Posts
- [ ] website
- [ ] FH '25
    - [ ] comp
    - [ ] e-motor throttle
    - [ ] throttle body
- [ ] picture frame
- [ ] arm assembly

**NOTE TO SELF**
Headers with the `'` character get cut 
for some reason, why is that? this should probably get fixed. 

**NOTE TO SELF**
Fix spacing on frontpage

## nix stuff
- [x] build ssg
- [x] build site (run ssg)
- [x] deploy site (run server)
- [x] fix:
    - [x] comments are just straight up not gonna work as is... nix store paths are immutable, so we can't push new comments
    - [x] the serve binary is running from it's nix store path, which doesn't even have the comment/challenge files...
    - [x] **probably the best solution is to a) run a database as another service, and have the server communicate through sockets or through TCP/IP on the same machine**
    - [x] **sqlite is probably the move lol**
    - [x] but we want to version control the comment files too...? do we need to do this in the same repo?
    - [x] can we just clone the repo to /var/www/andreano.dev or something? should we be serving from here anyway instead of from the derivation?
    - [x] probably we should NOT version control comments, at least in the same repository. Maybe the comment engine should be a separate project entirely, and run at comments.andreano.dev (CORS issues though lol)...
    - [x] also commit number doesn't even work lol, this needs to get fixed
    - [x] cube code was gitignored, fix that

## colors

- [x] fix color fading from default on homepage
- [x] cube color
- [ ] favicon colors
- [x] fenced code colors
    - done for now, but should figure out how to do custom themes
- [x] about page list colors


## server
- [x] routing table builds gets created on site build DEPRECATED
- [x] server supports webm locally
- [x] routing table includes index pages DEPRECATED

## Comment Engine
- [x] basic commenting
- [x] comments are per-page
- [x] comments are *only* on blog pages, will need new templating support for this (parse template from frontmatter?)
- [x] comment length: will need severe refactor to use `hyper` to get this going. 
- [x] challenge generation

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
- [x] adopt old posts to new format
    - [x] fh
    - [x] scraper thing
- [x] table of contents (sidecar?)
    can do this with while let
- [x] index page mobile formatting
- [x] blog post date and author 

## chrome problems
- [ ] math fonts
- [x] theme switch transitions
    - [x] calling this fixed, but it's not really. Nothing I can do here.

## mobile
- [ ] debounce theme switcher somehow

# After Done
- [ ] incremental rebuilds with file hashing
- [ ] collapsable headings as heading class
- [x] menu bar as component
- [x] include absolute styles from frontmatter
- [ ] include absolute scripts from frontmatter
- [x] include relative scripts from directory
- [x] include relative scripts from directory
- [x] md directory can have scripts and css, should get copied and included automatically
- [x] switch to sqlite backend or something for comments...

- [ ] for latex2mathml, image, consume closing tag
- [ ] rss feed
- [ ] text-only sitemap generation
- [ ] index page for subfolders in general? 
