# CubbyConnect

[![License](https://img.shields.io/github/license/CubbyTeam/CubbyConnect)](https://github.com/CubbyTeam/CubbyConnect/blob/main/LICENSE)
[![Github Action Server Audit](https://img.shields.io/github/workflow/status/CubbyTeam/CubbyConnect/Server%20Audit?label=Server%20Audit&logo=Github)](https://github.com/CubbyTeam/CubbyConnect/actions/workflows/server-audit.yml)
[![Github Action Server Build](https://img.shields.io/github/workflow/status/CubbyTeam/CubbyConnect/Server%20Build?label=Server%20Build&logo=Github)](https://github.com/CubbyTeam/CubbyConnect/actions/workflows/server-build.yml)
[![Github Action Server Clippy](https://img.shields.io/github/workflow/status/CubbyTeam/CubbyConnect/Server%20Clippy?label=Server%20Clippy&logo=Github)](https://github.com/CubbyTeam/CubbyConnect/actions/workflows/server-clippy.yml)
[![Github Action Server Coverage](https://img.shields.io/github/workflow/status/CubbyTeam/CubbyConnect/Server%20Coverage?label=Server%20Coverage&logo=Github)](https://github.com/CubbyTeam/CubbyConnect/actions/workflows/server-coverage.yml)
[![Github Action Server Fmt](https://img.shields.io/github/workflow/status/CubbyTeam/CubbyConnect/Server%20Fmt?label=Server%20Fmt&logo=Github)](https://github.com/CubbyTeam/CubbyConnect/actions/workflows/server-fmt.yml)
[![Github Action Server Test](https://img.shields.io/github/workflow/status/CubbyTeam/CubbyConnect/Server%20Test?label=Server%20Test&logo=Github)](https://github.com/CubbyTeam/CubbyConnect/actions/workflows/server-test.yml)
[![Codecov Coverage](https://img.shields.io/codecov/c/gh/CubbyTeam/CubbyConnect?logo=Codecov)](https://app.codecov.io/gh/CubbyTeam/CubbyConnect)

CubbyConnect is a voxel-based MMORPG server & client connecting each other.

## Features

- fast UDP connection
- secure TCP connection using TLS
- transfers data using protobuf
- pinging for heartbeat
- reconnection when internet is temporary disabled (in client)
- functional API that can be called in server & client
- connection to credential server for authentication
- version matching for compatability
- beautiful logging support
