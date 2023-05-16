# ls_oxside
![build](https://github.com/AG3NTSN0W/ls_oxide/actions/workflows/build.yml/badge.svg)

Automate repetitive tasks on any website.

## Examples

The examples assume you have `chromedriver` or `geckodriver` running on your system.

You can use Selenium (see instructions below) or you can use `chromedriver`/`geckodriver` directly by downloading the driver.
 - `chromedriver`: that matches your Chrome version <br>
    [https://chromedriver.chromium.org/downloads](https://chromedriver.chromium.org/downloads)

    Then run it like this:

        chromedriver

 - `geckodriver`: 
    [https://github.com/mozilla/geckodriver/releases](https://github.com/mozilla/geckodriver/releases)

    Then run it like this:

        geckodriver

### Example:

- using firefox `(default)` | `geckodriver`

    ```
    ls_oxside -t ./examples/wiki/wiki.yml
    ```

- using chrome | `chromedriver`

    ```
    ls_oxside -t ./examples/wiki/wiki.yml -c ./examples/wiki/config.yml
    ```


### Setting up Docker and Selenium

To install docker, see [https://docs.docker.com/install/](https://docs.docker.com/install/) (follow the SERVER section if you're on Linux, then look for the Community Edition)

Once you have docker installed, you can start the selenium server, as follows:

    docker run --rm -d -p 4444:4444 -p 5900:5900 --name selenium-server -v /dev/shm:/dev/shm selenium/standalone-chrome:4.1.0-20211123

For more information on running selenium in docker, visit
[docker-selenium](https://github.com/SeleniumHQ/docker-selenium)

## Build:

    cargo build --release

## Run Tests:

    cargo test  

### Example:

- using firefox `(default)` | `geckodriver`

    ```
    cargo run -- -t ./examples/wiki/wiki.yml
    ```

- using chrome | `chromedriver`

    ```
    cargo run -- -t ./examples/wiki/wiki.yml -c ./examples/wiki/config.yml
    ```        

