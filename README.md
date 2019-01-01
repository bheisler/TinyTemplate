<h1 align="center">TinyTemplate</h1>

<div align="center">Minimal Lightweight Text Templating</div>

<div align="center">
    <a href="https://bheisler.github.io/TinyTemplate/tinytemplate/index.html">API Documentation (master branch)</a>
</div>

<div align="center">
	<a href="https://travis-ci.org/bheisler/TinyTemplate">
        <img src="https://travis-ci.org/bheisler/TinyTemplate.svg?branch=master" alt="Travis-CI">
    </a>
</div>

TinyTemplate is a small, minimalistic text templating system with limited dependencies.

## Table of Contents
- [Table of Contents](#table-of-contents)
  - [Goals](#goals)
  - [Why TinyTemplate?](#why-tinytemplate)
  - [Quickstart](#quickstart)
  - [Contributing](#contributing)
  - [Maintenance](#maintenance)
  - [License](#license)

### Goals

 The primary design goals are:

 - __Small__: TinyTemplate deliberately does not support many features of more powerful template engines and is restricted to templates which are `&static str`'s.
 - __Simple__: TinyTemplate presents a minimal but well-documented user-facing API.
 - __Lightweight__: TinyTemplate has minimal required dependencies.

### Why TinyTemplate?

I created TinyTemplate after noticing that none of the existing template libraries really suited my
needs for Criterion.rs. Some had large dependency trees to support features that I didn't use. Some
required adding a build script to convert templates into code at runtime, in search of extreme
performance that I didn't need. Some have elaborate macro-based DSL's to generate HTML, where I just
wanted plain text with some markup. I just wanted something small and minimal with good 
documentation but there was nothing like that out there so I wrote my own.

### Quickstart

TODO: Write this

### Contributing

Thanks for your interest! Contributions are welcome.

Issues, feature requests, questions and bug reports should be reported via the issue tracker above.
In particular, becuase TinyTemplate aims to be well-documented, please report anything you find
confusing or incorrect in the documentation.

Code or documentation improvements in the form of pull requests are also welcome. Please file or
comment on an issue to allow for discussion before doing a lot of work, though.

For more details, see the [CONTRIBUTING.md file](https://github.com/bheisler/TinyTemplate/blob/master/CONTRIBUTING.md).

### Maintenance

TinyTemplate is currently maintained by Brook Heisler (@bheisler).

### License

TinyTemplate is dual-licensed under the Apache 2.0 license and the MIT license.
