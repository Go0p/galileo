---
sidebar_label: "Self-hosted V6 Swap API"
description: Unlock the potential of Self Hosted Jupiter Swap API for tailored trading solutions and independence from public API limits.
title: "Self-hosted V6 Swap API"
---

<head>
    <title>Self Hosted Jupiter Swap API - Personalized Infrastructure</title>
    <meta name="twitter:card" content="summary" />
</head>


Jupiter provides the ability for advanced users can run a self-hosted Jupiter Swap API. You can download the [jupiter-swap-api here](https://github.com/jup-ag/jupiter-swap-api/releases).

Mission-critical use cases, like liquidations and oracles, can deploy their own API servers relying on their own RPC nodes to entirely decouple their systems from Jupiter infrastructure.

Integrators load is no longer restricted by the public API rate limits.

## Prerequisites

A dedicated or shared Solana RPC node: **optional** but recommended with the [Yellowstone gRPC plugin](https://github.com/rpcpool/yellowstone-grpc) access.

The following RPC providers can provide a RPC node with the geyser plugin:

- [Triton](https://triton.one)
- [Helius](https://docs.helius.dev) Contact Helius on [Discord](https://discord.com/invite/6GXdee3gBj)
- [Shyft](https://shyft.to/#solana_grpc_streaming_service) Contact Shyft on [Discord](https://discord.gg/8JyZCjRPmr)
- [Solana Tracker](https://www.solanatracker.io/solana-rpc)

## Usage

To start the API server:

`RUST_LOG=info ./jupiter-swap-api --rpc-url <RPC-URL> --yellowstone-grpc-endpoint <GRPC-ENDPOINT> --yellowstone-grpc-x-token <X-TOKEN>`

For instance, if you used Triton and your RPC url is https://supersolnode.jup/91842103123091841, the arguments would be `--rpc-url https://supersolnode.jup/91842103123091841 --yellowstone-grpc-endpoint https://supersolnode.jup --yellowstone-grpc-x-token 91842103123091841`

It is also possible to run the API in poll mode (heavy for nodes and it is not recommended). It will periodically poll the Solana RPC node for accounts rather than listening with the Yellowstone gRPC endpoint:

`RUST_LOG=info ./jupiter-swap-api --rpc-url <RPC-URL>`

For others options, use `--help`:

`./jupiter-swap-api --help`

Once the API server is ready, it will open a HTTP server at `0.0.0.0:8080`.


The jupiter-swap-api is identical to the public Jupiter Swap API so all the documentation applies [Swap API](/docs/old/apis/swap-api), replacing the api URL `https://quote-api.jup.ag/v6` with `http://127.0.0.1:8080`.

## Market Cache

The Jupiter self hosted Swap API relies on the market cache https://cache.jup.ag/markets?v=3 maintained by the Jupiter team, as a snapshot of all the relevant markets after liquidity filtering.

To pick up those new markets the api has to be restarted. The cache is updated every 30 minutes.

This is the only reliance on Jupiter infrastructure.

## Adding New Markets (Without Restart)

To pick up new markets without restart, you can set `--enable-add-market` when starting the Jupiter self hosted Swap API. This way, you will see a new endpoint at `/add-market`. To add a new market without restarting the API, you can post to this endpoint. For example, let's say you have a new market on Raydium AMM, you will have to post the following payload to this endpoint:

```
{
  "address": "EzvDheLRnPjWy3S29MZYEi5qzcaR1WR5RNS8YhUA5WG5",
  "owner": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
  "params": { // Optional
    "serumAsks":"Ac8Hoi4LBbJfG4pCEUu2sS3jkmNrZBv6tbdmEnxAkRsK",
    "serumBids":"CF1NyAZjWqi8t9WZ7pSiqCiTSr3taZ94EW44AjyZRsnY",
    "serumCoinVaultAccount":"65LDE8k8WqhgrZy6NDsVQxGuUq3r8fT8bJunt5WPAZAk",
    "serumEventQueue":"1Xpk12GqjPLS8bkL8XVRHc6nrnunqcJhDha9jUq6Ymc",
    "serumPcVaultAccount":"AKATaDtSNPc5HemQCJzhph7o76Q1ndRHyKwai5C4wFkR",
    "serumVaultSigner":"7xookfS7px2FxR4JzpB3bT9nS3hUAENE4KsGaqkM6AoQ"
  },
  "addressLookupTableAddress":"5tVPTN4afHxuyS8CNCNrnU7WZJuYeq5e2FvxUdCMQG7F" // Optional
}
```

To derive the params, you can look up the [Serum documentation](https://github.com/project-serum/serum-dex/blob/master/dex/src/state.rs#L293-L343).

## MacOS

On MacOS you will see this error message:

`“jupiter-swap-api” can’t be opened because Apple cannot check it for malicious software.`

Go to System Settings and click on "Open Anyway":

![](@site/static/img/docs/jupiter-swap-api-open-anyway.png)

## Advanced

If a set of AMMs is never needed for routing, they can be removed before starting the api to reduce load.

Create a market-cache excluding the program you want to remove, Openbook for this example:

```shell
curl "https://cache.jup.ag/markets?v=3" -o market-cache.json
jq 'map(select(.owner != "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX"))' market-cache.json > market-cache-no-openbook.json
```

Then:

`RUST_LOG=info ./jupiter-swap-api --market-cache market-cache-no-openbook.json ...`

This will start the API server without Openbook as part of routing. You can also remove individual market as well.

## Paid Hosted APIs

We are working with some Solana RPC partners in the ecosystem as well so that you can get a paid hosted API ran by them.

- QuickNode: https://marketplace.quicknode.com/add-on/metis-jupiter-v6-swap-api
- Reach out to Triton: [Triton](https://t.me/SteveCleanBrook)



./jupiter-swap-api --help
Usage: jupiter-swap-api [OPTIONS] --rpc-url <RPC_URL>

Options:
      --market-cache <MARKET_CACHE>
          Jupiter europa URL, file path or remote file path, check production Jupiter cache for format https://cache.jup.ag/markets?v=4. Will default to the associated market mode default when not specified Note: the params field is required for some AMMs and is AMM type specific
          
          [env: MARKET_CACHE=]

      --market-mode <MARKET_MODE>
          Switch between market modes, file and remote will not receive new markets from Europa
          
          [env: MARKET_MODE=]
          [default: europa]
          [possible values: europa, remote, file]

      --rpc-url <RPC_URL>
          RPC URL for polling and fetching user accounts
          
          [env: RPC_URL=]

      --secondary-rpc-urls <SECONDARY_RPC_URLS>...
          Secondary RPC URLs used for some RPC calls
          
          [env: SECONDARY_RPC_URLS=]

  -e, --yellowstone-grpc-endpoint <YELLOWSTONE_GRPC_ENDPOINT>
          Yellowstone gRPC endpoint e.g. https://jupiter.rpcpool.com
          Make sure your config.json has high thresholds to accomodate the large amount of accounts the swap-api will subscribe to
          https://github.com/rpcpool/yellowstone-grpc/blob/a0bfd8a940e00a7cb2b8153d5462677ac9f70700/yellowstone-grpc-geyser/config.json
          
          i.e. config.json
          {
              "grpc": {
                  "address": "0.0.0.0:<grpc_port>",
                  "channel_capacity": "1_000_000",
                  "max_decoding_message_size": "1_000_000_000",
                  "unary_concurrency_limit": "100"
              },
             "libpath": "./libyellowstone_grpc_geyser.so",
              "log": {
                  "level": "info"
              },
              "prometheus": {
                  "address": "0.0.0.0:<prometheus_exporter_port>"
              }
          }
          
          [env: YELLOWSTONE_GRPC_ENDPOINT=]

  -x, --yellowstone-grpc-x-token <YELLOWSTONE_GRPC_X_TOKEN>
          Yellowstone gRPC x token, the token after the hostname
          
          [env: YELLOWSTONE_GRPC_X_TOKEN=]

      --yellowstone-grpc-enable-ping
          Enable pinging the grpc server, useful for a load balanced Yellowstone GRPC endpoint https://github.com/rpcpool/yellowstone-grpc/issues/225
          
          [env: YELLOWSTONE_GRPC_ENABLE_PING=]

      --yellowstone-grpc-compression-encoding <YELLOWSTONE_GRPC_COMPRESSION_ENCODING>
          [env: YELLOWSTONE_GRPC_COMPRESSION_ENCODING=]
          [default: gzip]
          [possible values: none, gzip, zstd]

      --snapshot-poll-interval-ms <SNAPSHOT_POLL_INTERVAL_MS>
          Interval after which AMMs related account should be fetched, in yellowstone grpc mode, there will be a periodic poll to snapshot the confirmed state of AMM accounts Default to 200 ms for poll mode and 30000 ms for yellowstone grpc mode
          
          [env: SNAPSHOT_POLL_INTERVAL_MS=]

      --enable-external-amm-loading
          Enable loading external AMMs from keyedUiAccounts in swap related endpoints
          
          [env: ENABLE_EXTERNAL_AMM_LOADING=]

      --disable-swap-cache-loading
          Disable loading caches necessary for swap related features to function properly, such as address lookup tables... This is useful for quote only APIs
          
          [env: DISABLE_SWAP_CACHE_LOADING=]

      --allow-circular-arbitrage
          Allow arbitrage quote and swap, where input mint is equal to output mint
          
          [env: ALLOW_CIRCULAR_ARBITRAGE=]

      --sentry-dsn <SENTRY_DSN>
          Sentry DSN to send error to
          
          [env: SENTRY_DSN=]

      --dex-program-ids <DEX_PROGRAM_IDS>...
          List of DEX program ids to include, other program ids won't be loaded, you can get program ids from https://quote-api.jup.ag/v6/program-id-to-label
          
          [env: DEX_PROGRAM_IDS=]

      --exclude-dex-program-ids <EXCLUDE_DEX_PROGRAM_IDS>...
          List of DEX program ids to exclude, from all program ids, excluded program ids won't be loaded, you can get program ids from https://quote-api.jup.ag/v6/program-id-to-label
          
          [env: EXCLUDE_DEX_PROGRAM_IDS=]

      --filter-markets-with-mints <FILTER_MARKETS_WITH_MINTS>...
          List of mints to filter markets to include, markets which do not have at least 2 mints from this set will be excluded
          
          [env: FILTER_MARKETS_WITH_MINTS=]

  -H, --host <HOST>
          The host
          
          [env: HOST=]
          [default: 0.0.0.0]

  -p, --port <PORT>
          A port number on which to start the application
          
          [env: PORT=]
          [default: 8080]

  -s, --expose-quote-and-simulate
          Enable the /quote-and-simulate endpoint to quote and simulate a swap in a single request
          
          [env: EXPOSE_QUOTE_AND_SIMULATE=]

      --enable-deprecated-indexed-route-maps
          Enable computating and serving the /indexed-route-map Deprecated and not recommended to be enabled due to the high overhead
          
          [env: ENABLE_DEPRECATED_INDEXED_ROUTE_MAPS=]

      --enable-diagnostic
          Enable the /diagnostic endpoint to quote
          
          [env: ENABLE_DIAGNOSTIC=]

      --enable-add-market
          Enable the /add-market endpoint to hot load a new market
          
          [env: ENABLE_ADD_MARKET=]

      --enable-tokens
          Enable the /tokens endpoint to list all tradable tokens, deprecated will be removed
          
          [env: ENABLE_TOKENS=]

      --enable-markets
          Enable the /markets endpoint to list all tradable markets, deprecated will be removed
          
          [env: ENABLE_MARKETS=]

      --metrics-port <METRICS_PORT>
          Port for Prometheus metrics endpoint `/metrics`
          
          [env: METRICS_PORT=]

      --enable-new-dexes
          Enable new dexes that have been recently integrated, new dexes: []
          
          [env: ENABLE_NEW_DEXES=]

      --environment <ENVIRONMENT>
          [env: ENVIRONMENT=]
          [default: production]

      --total-thread-count <TOTAL_THREAD_COUNT>
          Total count of thread to use for the jupiter-swap-api process
          
          [env: TOTAL_THREAD_COUNT=]
          [default: 3]

      --webserver-thread-count <WEBSERVER_THREAD_COUNT>
          Count of thread
          
          [env: WEBSERVER_THREAD_COUNT=]
          [default: 2]

      --update-thread-count <UPDATE_THREAD_COUNT>
          [env: UPDATE_THREAD_COUNT=]
          [default: 4]

      --loki-url <LOKI_URL>
          Loki url
          
          [env: LOKI_URL=]

      --loki-username <LOKI_USERNAME>
          Loki username
          
          [env: LOKI_USERNAME=]

      --loki-password <LOKI_PASSWORD>
          Loki password
          
          [env: LOKI_PASSWORD=]

      --loki-custom-labels <LOKI_CUSTOM_LABELS>...
          Custom labels to add to the loki metrics e.g. `APP_NAME=jupiter-swap-api,ENVIRONMENT=production`
          
          [env: LOKI_CUSTOM_LABELS=]

      --rtse-url <RTSE_URL>
          [env: RTSE_URL=]

      --geyser-streaming-chunk-count <GEYSER_STREAMING_CHUNK_COUNT>
          Number of chunks to use for geyser streaming
          
          [env: GEYSER_STREAMING_CHUNK_COUNT=]
          [default: 12]

      --yellowstone-grpc-setting-connect-timeout-ms <YELLOWSTONE_GRPC_SETTING_CONNECT_TIMEOUT_MS>
          Apply a timeout to connecting to the uri
          
          [env: YELLOWSTONE_GRPC_SETTING_CONNECT_TIMEOUT_MS=]

      --yellowstone-grpc-setting-buffer-size <YELLOWSTONE_GRPC_SETTING_BUFFER_SIZE>
          Sets the tower service default internal buffer size, default is 1024
          
          [env: YELLOWSTONE_GRPC_SETTING_BUFFER_SIZE=]

      --yellowstone-grpc-setting-http2-adaptive-window <YELLOWSTONE_GRPC_SETTING_HTTP2_ADAPTIVE_WINDOW>
          Sets whether to use an adaptive flow control. Uses hyper’s default otherwise
          
          [env: YELLOWSTONE_GRPC_SETTING_HTTP2_ADAPTIVE_WINDOW=]
          [default: true]
          [possible values: true, false]

      --yellowstone-grpc-setting-http2-keep-alive-interval-ms <YELLOWSTONE_GRPC_SETTING_HTTP2_KEEP_ALIVE_INTERVAL_MS>
          Set http2 KEEP_ALIVE_TIMEOUT. Uses hyper’s default otherwise
          
          [env: YELLOWSTONE_GRPC_SETTING_HTTP2_KEEP_ALIVE_INTERVAL_MS=]

      --yellowstone-grpc-setting-initial-connection-window-size <YELLOWSTONE_GRPC_SETTING_INITIAL_CONNECTION_WINDOW_SIZE>
          Sets the max connection-level flow control for HTTP2, default is 65,535
          
          [env: YELLOWSTONE_GRPC_SETTING_INITIAL_CONNECTION_WINDOW_SIZE=]
          [default: 8388608]

      --yellowstone-grpc-setting-initial-stream-window-size <YELLOWSTONE_GRPC_SETTING_INITIAL_STREAM_WINDOW_SIZE>
          Sets the SETTINGS_INITIAL_WINDOW_SIZE option for HTTP2 stream-level flow control, default is 65,535
          
          [env: YELLOWSTONE_GRPC_SETTING_INITIAL_STREAM_WINDOW_SIZE=]

      --yellowstone-grpc-setting-keep-alive-timeout-ms <YELLOWSTONE_GRPC_SETTING_KEEP_ALIVE_TIMEOUT_MS>
          Set http2 KEEP_ALIVE_TIMEOUT. Uses hyper’s default otherwise
          
          [env: YELLOWSTONE_GRPC_SETTING_KEEP_ALIVE_TIMEOUT_MS=]

      --yellowstone-grpc-setting-keep-alive-while-idle <YELLOWSTONE_GRPC_SETTING_KEEP_ALIVE_WHILE_IDLE>
          Set http2 KEEP_ALIVE_WHILE_IDLE. Uses hyper’s default otherwise
          
          [env: YELLOWSTONE_GRPC_SETTING_KEEP_ALIVE_WHILE_IDLE=]
          [possible values: true, false]

      --yellowstone-grpc-setting-tcp-keepalive-ms <YELLOWSTONE_GRPC_SETTING_TCP_KEEPALIVE_MS>
          Set whether TCP keepalive messages are enabled on accepted connections
          
          [env: YELLOWSTONE_GRPC_SETTING_TCP_KEEPALIVE_MS=]

      --yellowstone-grpc-setting-tcp-nodelay <YELLOWSTONE_GRPC_SETTING_TCP_NODELAY>
          Set the value of TCP_NODELAY option for accepted connections. Enabled by default
          
          [env: YELLOWSTONE_GRPC_SETTING_TCP_NODELAY=]
          [possible values: true, false]

      --yellowstone-grpc-setting-timeout-ms <YELLOWSTONE_GRPC_SETTING_TIMEOUT_MS>
          Apply a timeout to each request
          
          [env: YELLOWSTONE_GRPC_SETTING_TIMEOUT_MS=]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
