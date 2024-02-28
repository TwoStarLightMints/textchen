# textchen

**WARNING**: This project is still very much in its infancy. Many things are vulnerable to change, and some features are yet to be implemented. Please be patient at this time.

While there are many other text editors on the market (I wrote this one using [Helix](https://github.com/helix-editor/helix)), I wanted to try my hand at making one and take on the largest project I have done thus far.

Textchen is not necessarily meant to be a drop in replacement for such text editors like helix or vim, as I named it textchen is supposed to be small (-chen is the diminutive affix in German). You can definitely edit text files (I am writing this README in textchen right now) and there will be some more to come.

Textchen uses most of the traditional text editor motion keys such as h, j, k, and l, and it also uses i to enter insert mode, o to create a new line below the current one. More functionality to come.

Note: While I have put some work into making this program crossplatorm it has been primarily tested on linux (debian and arch). I have begun porting to Windows, and textchen should work properly on Windows machines.

## Install
You will need to have rust installed on your machine from the [rust language website](https://www.rust-lang.org).

```
$ git clone https://github.com/TwoStarLightMints/textchen.git
$ cd textchen
$ cargo install --path ./textchen
```

## Todos
See todo.todos for current plans