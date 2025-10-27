# static_atoms_rs

> Man.. wouldn't it be nice to just plug a bunch of HTML5 Compoonents into a static website and call it a day?
> Let me quickly build myself something using rust :)

A small Terminal Application that transforms a bunch of HTML Snippets into a static Website with nothing more than a bunch of Tags.
The Workflow is employing a simplified and modified atomic design model, and implementing a method to build a static website with it without any JS.

## Getting started
First, build this project using [rust cargo](https://rust-lang.org/tools/install/):
```sh
cargo build --release --bin static_atoms
```

There are a few things to do to properly structure the project:
* all your Pages reside in `<project_root>/pages`
* all your sections reside in `<project_root>/sections`
* your index file is found at `<project_root>/index.html`
* all your media sits in `<project_root>/media`
* your stylesheet sits in `<project_root>/style.css`

If you now want to embed the atom *my_embed.html* into your page, you do the following:
```html
<!DOCTYPE html>
<html lang="en">
    <head>
      ...
      <link rel="stylesheet" href="/style.css">
      ...
  </head>
  <body>
    <div class="content_center">
        <## my_embed>
    </div>
  </body>
</html>
```

afterwards you run the following in your project root:
```sh
static_atoms dist
```
Now your static website is in the folder `<project_root>/dist`

## Available Tags
There are more tags available for you tu use. Im going to list them here:
| Tag | Description |
|:---|:---|
| `<## embed_name>` | is a simple embed, that includes the HTML from `<project_root>/sections/embed_name.html` into wherever you try to embed it. |
| `<## embed_name[]>` | this is a folder embed. It includes the entire folder with this name `<project_root>/embed_name/*` one after another, in alphabetic fashion |
| `<## embed_name[..10]>` | the same as the folder embed, with the difference, that it only includes the first `10` entries of the selected folder, using the same sorting. |

## Available CLI Arguments
For a complete list run `static_atoms help`

| Command | Description |
|:---|:---|
|`static_atoms dist`| runs the main function and transforms all files in `<current_dir>/pages` into static pages within `<current_dir>/dist/pages` alongside all the necessary media and stylesheet.|
| `static_atoms dist --out=<path>` | runs the transformation, but instead places the pages files into `<path>/pages` alongside with all the necessary media and stylesheet. |
