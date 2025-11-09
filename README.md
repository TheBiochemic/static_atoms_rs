# static_atoms_rs

> Man.. wouldn't it be nice to just plug a bunch of HTML5 Compoonents into a static website and call it a day?
> Let me quickly build myself something using rust :)

A small Terminal Application that transforms a bunch of HTML Snippets into a static Website with nothing more than a bunch of Tags.
The Workflow is employing a simplified bottom-up Model (similar from what you know as the atomic pattern), and implementing a method to build a static website with it without any JS.
The tool is completely dependency free, and only uses rust's standard library features.

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
* additional files living at project_root are in `<project_root>/root`
* ~~your stylesheet sits in `<project_root>/style.css`~~ your global stylesheet now lives in the root directory

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
There are more tags available for you to use. Im going to list them here:
| Tag | Description |
|:--:|:---|
| `<## embed_name>` | is a simple embed, that includes the HTML from `<project_root>/sections/embed_name.html` into wherever you try to embed it. |
| `<## embed_name[]>` | this is a folder embed. It includes the entire folder with this name `<project_root>/embed_name/*` one after another, in alphabetic fashion |
| `<## embed_name[..10]>` | the same as the folder embed, with the difference, that it only includes the first `10` entries of the selected folder, using the same sorting. |
| `<## embed_name()>` | identical to a simple embed |
| `<## embed_name(variable="value")>` | a parametric embed, that does the same as a simple embed, except that variables with the name `variable` defined within the embedded object are being replaced by `value`. At this time time it does not support embeds as part of the value |
| `<## embed_name(var1="v1" var2="v2")>` | also a parametric embed, except with two variables, that are being replaced |
| `<## embed_name(var1="v1()" var2="<## other_embed>")>` | _New:_ you can now use brackets and other embeds within the value of the parameters. They get correctly resolved aswell. |
| `<## {variable}>` | a variable embed, that is being replaced with the value of `variable` passed into the current context by a parametric embed. If no variable has been found, it will be replaced by empty space |

## Predefined variables
There are a few variables, that are predefined, whenever a page is being parsed. They can always be used.
| Variable | Description |
|:--:|:---|
| `<## {_VERSION}>` | Gets replaced by the version of this Tool, such as `2025.4.1` |
| `<## {_APPNAME}>` | Gets replaced by the name of this Tool, such as `static_atoms_rs` |
| `<## {_APPLINK}>` | Gets replaced by a href link to the github of this tool, such as `<a href="..">static_atoms_rs</a>`
| `<## {_PAGES}>`| Gets replaced by an unordered list of href links to all available pages |

## Available CLI Arguments
For a complete list run `static_atoms help`

| Command | Description |
|:--:|:---|
|`static_atoms dist`| runs the main function and transforms all files in `<current_dir>/pages` into static pages within `<current_dir>/dist/pages` alongside all the necessary media and stylesheet.|
| `static_atoms dist --out=<path>` | runs the transformation, but instead places the pages files into `<path>/pages` alongside with all the necessary media and stylesheet. |
