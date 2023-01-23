# Drophost

Drophost is a simple maintaining tool for your `/etc/hosts` file. It allows you to easily configure your hosts file dynamically using a drop-in directory.

## Installation

### Using Cargo

```bash
cargo install drophost
```

### Arch Linux (AUR)

```bash
yay -S drophost
```

### Install from source

Download the source code from github and build it using cargo.

```bash
git clone https://github.com/KodiCraft/drophost.git
cd drophost
cargo build --release
```

Or use `cargo install`

```bash
git clone https://github.com/KodiCraft/drophost.git
cd drophost
cargo install --path .
```

## Usage

### Backup your current system hosts file

`drophost` comes with the ability of saving your current `/etc/hosts` file as a configuration file. This way, your current hosts file will automatically be used when you run `drophost`.

```bash
sudo drophost -b
```

This file will be named `10-old-config.conf` and will be located in the drop-in directory.

### Adding your own files

`drophost` will read files out of the `/etc/hosts.d` directory. Any files in the directory will be read and parsed. The files are read in alphabetical order, so you can use numbers to control the order in which they are read.

If you would like to test out your files, create a directory named `output` and treat it like your `/etc/` directory (i.e. add your configuration files to `output/hosts.d` and read the result from `output/hosts`). Then run `drophost` with the `-d` flag.

```bash
drophost -d
```

### Flags

Drophost comes with a few flags that can be used to customize its behavior.

* `-b` or `--backup` will backup your current hosts file to the drop-in directory.
* `-d` or `--dry-run` will run `drophost` without modifying your hosts file.
* `-w` or `--watch` will watch the drop-in directory for changes and automatically update your hosts file when a change is detected.
* `-h` or `--help` will display the help message.

You may also pass the `-l` or `--log-level` flag to set the log level. The default log level is `info`. The available log levels are `trace`, `debug`, `info`, `warn`, `error`, and `off`.

### File format

Files for `drophost` are fully compatible with the syntax of the `/etc/hosts` file, but they do add some additional features.

As a reminder, hosts can be added with the following syntax:

```conf
ip.address hostname
```

And comments can be inserted with the `#` character.

```conf
# This is a comment
```

#### Loud comments

Loud comments are comments that will display a warning in the logs when they are encountered. This allows you to debug your configuration files easily.

```conf
#=> This is a loud comment
```

#### Variables

`drophost` allows you to define variables in your configuration file and fetch them later. You can set a variable with the following syntax:

```conf
set variable_name = value
```

You can then fetch the value of a variable with the following syntax:

```conf
$variable_name
```

Variables can be used practically everywhere in your configuration file. For example, you can use them to define a hostname:

```conf
set hostname = my-hostname
127.0.0.1 $hostname
```

Or to define an IP address:

```conf
set ip = 127.0.0.1
$ip my-hostname
```

You can also unset variables with the following syntax:

```conf
unset variable_name
```

Variables are maintained between different configuration files. This means that you can define a variable in one file and use it in another.

Additionally, all environment variables that `drophost` is called with are automatically available as variables. For example, if you call `drophost` with the `HOSTNAME` environment variable set to `my-hostname`, you can use it in your configuration file:

```conf
127.0.0.1 $env_HOSTNAME
```

#### Conditionals

Conditionals allow you to include branching logic in your configuration files. They are defined with the following syntax:

```conf
if $value1 == $value2
    # Do something
else
    # Do something else
end
```

The `if` statement will evaluate the condition and execute the block if the condition is true. If the condition is false, the `else` block will be executed. If no `else` block is defined, the `if` block will be executed if the condition is true.

You may use variables or literal values in the condition. For example, you can use the following syntax to check if a variable isn't equal to "hello world":

```conf
if $variable != hello world
    # Do something
end
```

Note how there aren't any quotes, spaces are allowed in variable names and in literal values.

Since all variables are strings, you only have the `==` and `!=` operators available.

#### External conditions

Sometimes it can be useful to check if certain states are met that are not related to `drophost`'s logic. This is where the `try` syntax comes in handy.

You can `try` any of the following conditions:

* `file <path>`: Checks if a file exists at the given path.
* `var <name>`: Checks if a variable is defined. **Variables can be an empty string and be considered "defined"**
* `has <hostname>`: Checks if a hostname has been previously defined.

Additionally, if you compile the project with the `ping` feature, you can also `try` the following condition:

* `ping <hostname>`: Checks if a hostname is reachable.

**The releases in the Github Actions tab are never compiled with additional features!**

The `try` syntax is defined as follows:

```conf
try <condition> <argument>
    # Do something
else
    # Do something else
end
```

The `try` statement will evaluate the condition and execute the block if the condition is true. If the condition is false, the `else` block will be executed. If no `else` block is defined, the `try` block will be executed if the condition is true.

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for more details.

## Contributing

Help is always welcome! If you find a bug or have an idea for a new feature, please open an issue. If you want to contribute code, please open a pull request.

Make sure to write new tests for your code if it adds any new functionality! If you're not sure how to do that, feel free to ask for help.
