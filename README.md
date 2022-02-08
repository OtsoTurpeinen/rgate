# RGate
RGate my little project to take http server abstraction to logical extreme. Main goal is to allow use of any preprocessors you want to install to your unix enviorment when serving web pages. This means any templating engine or even output of shell script can be served.

## How to use
### Config.toml
Config.toml has example server config. This has basic details like server ports etc. I will later probly use .env and system env to make it easier for containers but currently this is how it is :)

### Preproc.toml
Preproc.toml is the meat of the operation. Only required field is the extension. This eliminates the need for extension in the adress bar. Supported properties:
* extension: what is the end of the file name that this applies to.
* command: what is ran in your unix shell. defaults to `cat`
* input_type: how the file is used by the command
⋅⋅* "pipe": (default) equilevant of using ` < file.php` at the end of shell command.
⋅⋅* "file":  equilevant of using ` file.php` at the end of shell command.
* priority: if there is conflict between preprocessors, one with lower number is selected. defaults to MAX `u16`

## Tested
Some of the tested 'preprocessors' so far are:
* cat - most common terminal tool to quickly view file contents.
* php - seems to work fine, argument passing needs to be implemented.
* figlet - cli for transforming text into ascii art.
* rant - cli for 'rant' language. currently requires "file" input_type to work.

## Future
I am building this with intent to build my small portfolio page. I absolutely intend to 'dog food' this project.

## Licence
I have not decided yet.