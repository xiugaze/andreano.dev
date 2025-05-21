---
title: "colophon"
---

- This website runs on a [NixOS](https://nixos.org/) system on my LAN. 
- This website is served by a custom web server behind a caddy reverse proxy. 
- This website is proxied by cloudflare. I don't like cloudflare in general but its preferable to exposing my ip address. 
- Most of this website is either handmade HTML or hand-written markdown, or a mix of the two. 
- All of the CSS and most of the javascript is handwritten. 
- This site is built using a custom static site generator that I wrote. It's not very good and it is very slow, but it works for me. when I run it all my fans spin up in my computer.
- This site is not really static. There's some javascript that runs client side, particularity for the 
theme switcher and the comment engine. 
- The source for this website and the static site generator is [on github](https://github.com/xiugaze/andreano.dev).
- This website has no cookies or analytics. The theme preference is stored in localstorage, that's it.


I used to build it through Hugo and serve through github pages but I didn't like that very much because I didn't really understand what was going on behind the scenes. 
Building the SSG from scratch totally alleviated the knowledge gap, but is objectively worse for user and developer experience. 
