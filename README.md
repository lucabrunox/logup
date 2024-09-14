# About outlog 

Outlog is a UNIX-style command that can be used to pipe stdout logs to a location on disk or in the cloud without the need of an agent, logrotate, systemd or other configuration files.

Outlog is resilient: it does buffering to temp files to prevent the application from ever blocking when writing to stdout. (Not implemented yet)

Outlog is transparent: it passes through the original stdout without any additional info or error messages.

## Use cases

Upload to AWS Logs:

```bash
# environment with region and credentials
$ echo foo | outlog --aws --aws-log-group-name '/test/foo'
foo
```

Upload to NewRelic:

```bash
$ export NEW_RELIC_API_KEY = "..."
$ echo foo | outlog --newrelic --newrelic-region EU
foo
```

Pipe stdout to disk files with log rotation, without the need to set up logrotate. (Not implemented yet)

## Installation ![](https://github.com/lucabrunox/outlog/actions/workflows/ci.yml/badge.svg)

To install in ~/.cargo/bin from git:

```bash
cargo install --git https://github.com/lucabrunox/outlog
```

## Command line usage

```bash
Usage: outlog [OPTIONS]

Options:
      --max-line-size <MAX_LINE_SIZE>
          Force flush without newline beyond the given size [default: 1000000]
      --max-memory-items <MAX_MEMORY_ITEMS>
          Max logs to keep in memory before dropping the incoming ones [default: 1000]
      --max-retries <MAX_RETRIES>
          Max retries before dropping a log [default: 100]
      --aws
          Enable uploading logs to AWS Logs
      --aws-log-group-name <AWS_LOG_GROUP_NAME>
          
      --aws-log-stream-name <AWS_LOG_STREAM_NAME>
          Log stream name [default: hostname]
      --newrelic
          Enable uploading logs to NewRelic
      --newrelic-region <NEW_RELIC_REGION>
          [env: NEW_RELIC_REGION] [possible values: US, EU]
      --newrelic-api-key <NEW_RELIC_API_KEY>
          [env: NEW_RELIC_API_KEY]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Roadmap

- [X] Send logs to AWS Logs
- [X] Buffering in-memory
- [X] Splitting by lines
- [ ] Read from file instead of just stdout
- [ ] Buffering on-disk
- [ ] Output to disk files with log rotation
- [ ] Compression
- [ ] Logging of outlog itself to disk
- [ ] Expose Prometheus endpoint of outlog itself
- [ ] Distributions
  - [ ] Cargo
  - [ ] Tar
  - [ ] Deb
  - [ ] Rpm
- [ ] Support more outputs
  - [ ] Cloud providers
  - [ ] Syslog
  - [ ] OTLP

## License

Outlog is licensed under the GPLv3: https://www.gnu.org/licenses/gpl-3.0.html#license-text

All contributions are welcome.