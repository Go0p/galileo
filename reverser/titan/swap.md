# Swap API

The Titan Swap API requires an API token to work. Please contact <info@titandex.io> or reach out to the team on telegram/discord.&#x20;

This API primarily allows for requesting quotes for token swaps and receiving a live stream of quotes with updated simulated results.

There are two primary layers of the API:

* The Remote Procedure Call (RPC) protocol, which defines what requests the client can make and what responses they should expect for each request.
* The Wire Protocol, which defines what messages can be sent by each side of the connection and how they are formatted.

## WebSocket Connections

The server **MAY** define a specific endpoint to connect to for WebSocket connections (e.g. `/api/v1/ws`).

All Titan API messages will be sent as `Binary` messages. Attempts to send `Text` messages **SHOULD** be ignored.

The client and server **SHOULD** support all valid non-data WebSocket message types (e.g. `Ping`, `Pong`, etc) appropriately.

### Protocol Negotiation

When initiating a WebSocket connection the server, the client **MUST** specify the protocol version and encoding schemes it supports in the [`Sec-WebSocket-Protocol`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Sec-WebSocket-Protocol) header. The values **SHOULD** be specified in order of preference, with the most preferred protocol and encoding scheme specified first.

All protocol values for version 1 begin with `v1.api.titan.ag`. The protocol may be suffixed with a plus (+) character followed by an encoding scheme.

Currently supported compression schemes are:

* `zstd` - Compression using [zstd](https://github.com/facebook/zstd).
* `brotli` - Compression using [brotli](https://github.com/google/brotli).
* `gzip` - Compression using [gzip](https://www.gzip.org/).

Valid protocol values for version 1 are:

* `v1.api.titan.ag`
* `v1.api.titan.ag+zstd`
* `v1.api.titan.ag+brotli`
* `vi.api.titan.ag+gzip`

If an encoding scheme is specified, the encoding is applied to all messages AFTER encoding and BEFORE decoding.

### Authentication

The server **MAY** require authentication from users in order to connect. Version 1 requires any authentication to occur up-front, before requests are made.

If authentication is required, the client **MUST** submit valid credentials via the `Authentication` header.

The standard authentication scheme will be Json Web Tokens (JWTs) submitted as one of the following:

* A `Bearer` token in the `Authorization` header (recommended).
* Via the `auth` query parameter for clients that cannot specify headers (e.g. browsers).

Submitted JWTs **MUST** be signed by an authority trusted by the servers and **MUST** include the following claims:

* `iss`: Issuer of the JWT
* `sub`: JWT subject, which should be a unique identifier for the user being authenticated.
* `aud`: JWT audience, must be `api.titan.ag`.
* `exp`: Token expiration time, connections will be refused if the specified time is in the past.
* `iat`: When the JWT was issued. The server **MUST** reject tokens with issue times in the future.

JWTs **MAY** include the following registered claims:

* `nbf`: If set, connections will be refused if the specified time is in the future.
* `jti`: If set and supported by the server, only one connection per unique `jti` value will be accepted.

JWTs **MAY** include the following custom claims.

* `https://api.titan.ag/upk_b58`: A Solana public key, encoded as a Base58 string. If set, this will be used as the user's public key for transaction generation. Any attempt by the user to submit a different key or change the default key to a different value will result in an error.

## Remote Procedure Call Interface

When a client connects to a Titan API server, the basic way it interacts is by making requests with a given set of parameters, and expecting a result (success or error) for that request.

| Procedure          | Parameters           | Response Type           | Stream Type |
| ------------------ | -------------------- | ----------------------- | ----------- |
| GetInfo            | GetInfoRequest       | ServerInfo              | None        |
| NewSwapQuoteStream | SwapQuoteRequest     | QuoteSwapStreamResponse | SwapQuotes  |
| StopStream         | StopStreamRequest    | StopStreamResponse      | None        |
| GetVenues          | GetVenuesRequest     | VenueInfo               |             |
| ListProviders      | ListProvidersRequest | `ProviderInfo[]`        |             |

Functionally, a connection to the API server could be modeled as something like:

```typescript
interface TitanAPI {
  getInfo(params: GetInfoRequest): Promise<ServerInfo>;
  newSwapQuoteStream(
    params: SwapQuoteRequest,
  ): Promise<[QuoteSwapStreamResponse, Stream<SwapQuotes>]>;
  stopStream(params: StopStreamRequest): Promise<StopStreamResponse>;
  getVenues(params: GetVenuesRequest): Promise<VenueInfo>;
  listProviders(params: ListProvidersRequest): Promise<ProviderInfo[]>;
}
```

## Data Format

The basic data format utilized for serialization of messages is [MessagePack](https://msgpack.org/).

Objects/structs **MUST** be encoded as maps unless otherwise specified. This is to allow for additional fields to be added without breaking compatibility with previous versions of the protocol.

Field names in objects/structs are encoded in `camelCase` unless otherwise specified.

### Optional Data

If a value is optional, its type will be represented in Rust as `Option<T>` and TypeScript as `T?` or `T | null`;

If the optional data is a field in an object/struct, it **MAY** be omitted from the serialized map if missing. Otherwise a missing optional value **SHOULD** be encoded as `nil` (`0xc0`). This will be decoded as `None` in Rust and `null` in TypeScript.

### Simple Enumerations

Simple enumerations, which are those without associated data, are encoded as strings matching the names of the enumeration values exactly.

For example:

```rust
enum SwapMode {
  ExactIn,
  ExactOut,
}
```

The above enumeration values would be encoded as the strings `"ExactIn"` and `"ExactOut"`.

### Complex Enumerations

Complex enumerations, which are those with associated data, are encoded as single-value maps, mapping the name of the enumeration value to the associated data.

If the associated data consists of a single item, the value is that data.

If the associated data consists of multiple items, the value is an array containing the associated data items.

For example, given the following enumeration:

```rust
struct Request2Data {
  id: u32,
  amount: u64,
}

enum Complex {
  Request1(String),
  Request2(Request2Data),
  Request3(u32, u32),
}
```

The following are valid encodings of `Complex`, presented as JSON for readability:

```json
{ "Request1": "hello" }
{ "Request2": {"id": 1, "amount": 34} }
{ "Request3": [3, 4] }
```

The above enumeration could be represented in TypeScript as a union:

```typescript
interface Request2Data {
  id: number;
  amount: number;
}

type Complex =
  | { Request1: String }
  | { Request2: Request2Data }
  | { Request3: [number, number] };
```

### Binary Data

Binary data **SHOULD** be encoded using one of the `bin` formats defined in MessagePack.

In Rust, the following types are encoded as binary data unless otherwise specified:

* `Vec<u8>` for a variable-sized byte array.
* `[u8; N]` for a fixed sized byte array, where N is an integer giving the number of bytes.

In TypeScript, the following types are encoded as binary:

* `Uint8Array`
* `ArrayBuffer`

TypeScript has no easy way of specifying the size of a byte array, so please look at the Rust types for information regarding that.

## Wire Protocol Type Definitions

### Common Types

#### **Pubkey**

Solana public keys are encoded as 32-byte binary data. Pubkeys **SHOULD** be encoded using the `bin 8` format for size. Thus, all public keys will start with `c4 20` followed by the public key data.

For example, the WSOL public key `So11111111111111111111111111111111111111112` would be encoded as (spaces added for legibility):

`c4 20 069b8857feab8184fb687f634618c035dac439dc1aeb3b5598a0f00000000001`

In TypeScript, this would be represented as a `Uint8Array` of length 32.

```typescript
// Type alias for public keys to differentiate them from other
// binary data.
type Pubkey = Uint8Array;
```

#### **AccountMeta**

Solana Account metadata is commonly used for instruction data, so it has a custom encoding with shorter field names to save space.

The Rust definition for the encoded form is:

```rust
struct AccountMeta {
  p: Pubkey, // public key
  s: bool,   // is_signer
  w: bool,   // is_writable
}
```

In TypeScript:

```typescript
interface AccountMeta {
  p: Pubkey;
  s: boolean;
  w: boolean;
}
```

#### **Instruction**

Similar to `AccountMeta`, instructions are also heavily used and thus have a custom encoding with shorter field names to save space.

```rust
struct Instruction {
  p: Pubkey,            // program_id
  a: Vec<AccountMeta>,  // accounts
  d: Vec<u8>,           // data
}
```

```typescript
interface Instruction {
  p: Pubkey;
  a: AccountMeta[];
  d: Uint8Array;
}
```

#### **SwapMode**

Represents how an amount in a swap request should be interpreted.

If `ExactIn`, the user is requesting that an exact number of input tokens should be swapped. In this case, any slippage will occur on the output token.

If `ExactOut`, the user is requesting that an exact number of output tokens should be obtained from the swap. In this case, any slippage will occur on the input token. Not all providers support `ExactOut` swaps.

```rust
enum SwapMode {
  ExactIn,
  ExactOut
}
```

```typescript
enum SwapMode {
  ExactIn = "ExactIn",
  ExactOut = "ExactOut",
}
```

### Client Requests

Requests from the client are represented as follows:

```rust
struct ClientRequest {
  id: u32,
  data: RequestData,
}

enum RequestData {
  GetInfo(GetInfoRequest),
  NewSwapQuoteStream(SwapQuoteRequest),
  StopStream(StopStreamRequest),
  GetVenues(GetVenuesRequest),
  ListProviders(ListProvidersRequest),
}
```

```typescript
interface ClientRequest {
  id: number;
  data: RequestData;
}

type RequestData =
  | { GetInfo: GetInfoRequest }
  | { NewSwapQuoteStream: SwapQuoteRequest }
  | { StopStream: StopStreamRequest }
  | { GetVenues: GetVenuesRequest }
  | { ListProviders: ListProvidersRequest };
```

These map to the following request data:

* `GetInfo` -> `GetInfoRequest`
* `NewSwapQuoteStream` -> `SwapQuoteRequest`
* `StopStream` -> `SwapStreamRequest`
* `GetVenues` -> `GetVenuesRequest`
* `ListProviders` -> `ListProvidersRequest`

#### **`GetInfoRequest`**

This request currently has no options, so the associated data is represented as an empty object. This is done to allow expansion in the future.

The server **SHOULD** ignore any unknown fields.

```rust
struct GetInfoRequest { }
```

```typescript
interface GetInfoRequest {}
```

#### **`SwapQuoteRequest`**

Parameters for requesting quotes for a swap.

```rust
struct SwapQuoteRequest {
  // Parameters for the swap.
  swap: SwapParams,
  // Parameters for transaction generation.
  transaction: TransactionParams,
  // Parameters for the stream of quote updates.
  update: Option<QuoteUpdateParams>,
}

struct SwapParams {
  /// Address of the input mint of the swap.
  inputMint: Pubkey,
  /// Address of the desired output token for the swap.
  outputMint: Pubkey,
  /// Raw number of tokens to swap, not scaled by decimals.
  amount: u64,
  /// Swap mode for how the amount should be interpreted.
  ///
  /// Either ExactIn or ExactOut, defaults to ExactIn.
  /// ExactOut is for supporting use cases where you need an exact output
  /// token amount, like payments. In this case, slippage would be on the input
  /// token. May not always be supported.
  swapMode: Option<SwapMode>,
  /// Allowed slippage in basis points.
  slippageBps: Option<u16>,
  /// If set, constrain quotes to the given set of DEXes.
  ///
  /// Note: setting both `dexes` and `exclude_dexes` may result in excluding all
  /// dexes, resulting in no routes.
  dexes: Option<Vec<String>>,
  /// If set, exclude the following DEXes when determining routes.
  ///
  /// Note: setting both `dexes` and `exclude_dexes` may result in excluding all
  /// dexes, resulting in no routes.
  excludeDexes: Option<Vec<String>>,
  /// If set to true, only direct routes between the input and output mint will
  /// be considered.
  onlyDirectRoutes: Option<bool>,
  /// If set to true, only quotes with transactions that fit within the size
  /// constraint are returned.
  addSizeConstraint: Option<bool>,
  /// The size constraint to use when `addSizeConstraint` is set.
  /// Default is set by the server, but is normally set to a value slightly less
  /// than the maximum transaction size of 1232 to allow room for additional
  /// instructions, such as compute budgets and fee accounts.
  sizeConstraint: Option<u32>,
  /// If set, limit quotes to the given set of provider IDs.
  providers: Option<Vec<String>>,
  /// If set, limit total number of accounts used by routes to the specified value.
  ///
  /// If not set, any number of accounts that still allows for an executable transaction is
  /// allowed. As of writing, this is 256 accounts.
  ///
  /// Available Since: v1.1
  pub accountsLimitTotal: Option<u16>,
  /// If set, limit total number of writable accounts used by routes to the specified value.
  ///
  /// If not set, any number of accounts that still allows for an executable transaction is
  /// allowed. As of writing, this is 64 writable accounts.
  ///
  /// Available Since: v1.1
  pub accountsLimitWritable: Option<u16>,
}

struct TransactionParams {
  /// Public key of the user requesting the swap, needed for transaction generation.
  ///
  /// NOTE: Setting this to a read-only system account will result in simulations
  /// failing and no quotes being returned.
  userPublicKey: Pubkey,
  /// If true, close the input token account as part of the transaction.
  closeInputTokenAccount: Option<bool>,
  /// If true, an idempotent ATA will be added to the transactions, if supported
  /// by the providers.
  createOutputTokenAccount: Option<bool>,
  /// The address of a token account for the output mint that will be used token
  /// collect fees.
  /// This account must already exist, or the user must add the ATA creation
  /// instruction themselves.
  feeAccount: Option<Pubkey>,
  /// Fee amount to take, in basis points.
  ///
  /// If not specified, default fee for the requester is used.
  feeBps: Option<u16>,
  /// Whether the fee should be taken in terms of the input mint.
  /// Default is false, in which case the fee is taken in terms of the output mint.
  feeFromInputMint: Option<bool>,
  /// Address of the token account into which to place the output of the swap.
  /// If not specified, the funds will be deposited into an ATA associated with the user's
  /// wallet.
  outputAccount: Option<Pubkey>,
}

struct QuoteUpdateParams {
  /// How often the server should send updates for this quote request, in milliseconds.
  ///
  /// If not specified, the server default will be used.
  intervalMs: Option<u64>,
  /// Maximum number of quotes per update the server should return. If more quotes are available,
  /// the worst will be filtered out, based on amount in/out depending on swap mode.
  ///
  /// If not specified, the server default will be used.
  numQuotes: Option<u32>,
}
```

In TypeScript:

```typescript
interface SwapQuoteRequest {
  // Parameters for the swap.
  swap: SwapParams;
  // Parameters for transaction generation.
  transaction: TransactionParams;
  // Parameters for the stream of quote updates.
  update?: QuoteUpdateParams;
}

interface SwapParams {
  // Address of input mint for the swap.
  inputMint: Pubkey;
  // Address of output mint of the swap.
  outputMint: Pubkey;
  // Raw number of tokens to swap, not scaled by decimals.
  // Whether this is in terms of the input or
  // output depends on the value of swapMode.
  amount: number;
  // Whether amount is in terms of inputMint or outputMint.
  // Defaults to ExactIn.
  swapMode?: SwapMode;
  // Maximum allowed slippage, in basis points.
  slippageBps?: number;
  // If set, constrain quotes to the given set of DEXes.
  dexes?: string[];
  // If set, exclude the following DEXes when determining routes.
  excludeDexes?: string[];
  // If true, only direct routes between the input and output mint will be considered.
  onlyDirectRoutes?: boolean;
  // If set to true, will request that quote providers restrict their quotes token
  // transactions that will fit within the size constraint.
  addSizeConstraint?: boolean;
  // The size constraint to use when `addSizeConstraint` is set.
  // Default is set by the server, but is normally set to a value slightly less
  // than the maximum transaction size of 1232 to allow room for additional
  // instructions, such as compute budgets and fee accounts.
  sizeConstraint?: number;
  // If set, limit quotes to the given set of provider IDs.
  providers?: string[];
  // If set, limit total number of accounts used by routes to the specified value.
  //
  // If not set, any number of accounts that still allows for an executable transaction is
  // allowed. As of writing, this is 256 accounts.
  //
  // Available Since: v1.1
  accountsLimitTotal?: number;
  // If set, limit total number of writable accounts used by routes to the specified value.
  //
  // If not set, any number of accounts that still allows for an executable transaction is
  // allowed. As of writing, this is 64 writable accounts.
  //
  // Available Since: v1.1
  accountsLimitWritable?: number;
}

interface TransactionParams {
  // Public key of the user requesting the swap, needed for transaction generation.
  //
  // NOTE: Setting this to a read-only system account will result in simulations
  // failing and no quotes being returned.
  userPublicKey: Pubkey;
  // If true, close the input token account as part of the transaction.
  closeInputTokenAccount?: boolean;
  // If true, an idempotent ATA will be added to the transactions, if supported
  // by the providers.
  createOutputTokenAccount?: boolean;
  // The address of a token account for the output mint that will be used
  // to collect fees.
  // This account must already exist, or the user must add the ATA creation
  // instruction themselves.
  feeAccount?: Pubkey;
  // Fee amount to take, in basis points.
  //
  // If not specified, default fee for the requester is used.
  feeBps?: number;
  // Whether the fee should be taken in terms of the input mint.
  // Default is false, in which case the fee is taken in terms of the output mint.
  feeFromInputMint?: boolean;
  // Address of the token account into which to place the output of the swap.
  // If not specified, the funds will be deposited into an ATA associated with the user's
  // wallet.
  outputAccount?: Pubkey;
}

interface QuoteUpdateParams {
  // How often the server should send updates for this quote request, in milliseconds.
  //
  // If not specified, the server default will be used.
  intervalMs?: number;
  // Maximum number of quotes to return.
  numQuotes: Option<u32>;
}
```

#### **`StopStreamRequest`**

Parameters for requesting the end of a stream of data.

Once sent to the server, the client **MAY** receive one or more `StreamData` messages for the stream if any were already queued for sending, followed by a `StreamEnd` message indicating that no more data will be sent.

```rust
struct StopStreamRequest {
  /// ID of the stream to stop.
  id: u32,
}
```

In TypeScript:

```typescript
interface StopStreamRequest {
  // ID of the stream to stop.
  id: number;
}
```

#### **`GetVenuesRequest`**

Parameters for requesting the list of known venues used by the various providers.

```rust
struct GetVenuesRequest {
  // Whether to include program ID for each venue.
  includeProgramIds: Option<bool>,
}
```

In TypeScript:

```typescript
interface GetVenuesRequest {
  includeProgramIds?: bool;
}
```

**`ListProvidersRequest`**

Parameters for requesting the list of configured quote providers.

```rust
struct ListProvidersRequest {
  // Whether or not to include icon URLs for each provider.
  // By default, icons are not included.
  includeIcons: Option<bool>,
}
```

In TypeScript:

```typescript
interface ListProvidersRequest {
  // Whether or not to include icon URLs for each provider.
  // By default, icons are not included.
  includeIcons?: bool;
}
```

### Server Messages

Responses from the server are represented as follows:

```rust
enum ResponseData {
    /// Response for a GetInfo request.
    GetInfo(ServerInfo),
    /// Response for a NewSwapQuoteStream request.
    NewSwapQuoteStream(QuoteSwapStreamResponse),
    /// Successful response to a StopStream request.
    StreamStopped(StopStreamResponse),
    /// Successful response to a GetVenues request.
    GetVenues(VenueInfo),
    /// Successful response to a ListProviders request.
    ListProvider(Vec<ProviderInfo>),
}

/// Types of data that can be streamed to the client.
///
/// Included in a [`StreamStart`] message so that the client may prepare for the incoming data.
enum StreamDataType {
    /// Stream contains [`SwapQuotes`] as the data values.
    SwapQuotes,
    // May be expanded in the future.
}


/// Notification that a new stream has been started by the server.
///
/// Returned as part of a successful response so that the client can allocate resources for the
/// stream.
struct StreamStart {
    /// ID of the new stream.
    ///
    /// This ID will be present in [`StreamData`] and [`StreamEnd`] messages for this stream.
    pub id: u32,
    /// The type of data that will be sent in this stream.
    pub dataType: StreamDataType,
}

/// Data packet for a stream.
struct StreamData {
    /// ID of the stream.
    pub id: u32,
    /// Sequence number of this data packet.
    pub seq: u32,
    /// Data payload.
    pub payload: StreamDataPayload,
}

/// Notification that a stream has closed and will no longer receive data.
struct StreamEnd {
    /// ID of the stream that has ended.
    pub id: u32,
    /// If the stream ended due to an error, contains the code denoting the specific error that
    /// occurred.
    pub errorCode: Option<u32>,
    /// If the stream ended due to an error, contains a message explaining what happened.
    pub errorMessage: Option<String>,
}

/// A response to a successful request.
struct ResponseSuccess {
    /// Identifier of the request that triggered this response.
    pub requestId: u32,
    /// The response data.
    pub data: ResponseData,
    /// If this request starts a new stream, contains information about the
    /// stream ID and its data type.
    pub stream: Option<StreamStart>,
}

/// A response to a request that resulted in an error.
struct ResponseError {
    /// Identifier of the request that triggered this response.
    requestId: u32,
    /// A numeric error code representing the specific error that occurred.
    code: u32,
    /// An message describing the error.
    message: String,
}

/// Payload for a [`StreamData`] message.
enum StreamDataPayload {
    /// Payload contains a set of quotes for a swap request.
    SwapQuotes(SwapQuotes),
    // May be expanded in the future.
}

/// A message sent by the server to the client.
enum ServerMessage {
    /// Successful response to a request, which may optionally start a stream.
    Response(ResponseSuccess),
    /// An error response to a request.
    Error(ResponseError),
    /// Data for a stream.
    StreamData(StreamData),
    /// Notification that a stream has ended.
    StreamEnd(StreamEnd),
}
```

In TypeScript:

```typescript
type ResponseData =
  | { GetInfo: ServerInfo }
  | { NewSwapQuoteStream: QuoteSwapStreamResponse }
  | { StreamStopped: StopStreamResponse }
  | { GetVenues: VenueInfo }
  | { ListProviders: ProviderInfo[] };

enum StreamDataType {
  SwapQuotes = "SwapQuotes",
}

interface StreamStart {
  // ID of the stream. All StreamData and StreamEnd messages for this stream will
  // be tagged with this ID.
  id: number;
  // Data type that will be encoded in the stream.
  dataType: StreamDataType;
}

interface ResponseSuccess {
  // Identifier of the request that triggered this response.
  requestId: number;
  // The response data.
  data: ResponseData;
  // If the request started a new stream, contains information about the stream.
  stream?: StreamStart;
}

interface ResponseError {
  // Identifier of the request that triggered this response.
  requestId: number;
  // A numeric error code representing the specific error that occurred.
  code: number;
  // A message describing the error.
  message: string;
}

type StreamDataPayload = { SwapQuotes: SwapQuotes };

interface StreamData {
  // ID of the stream.
  id: number;
  // Sequence number of this data packet.
  seq: number;
  // Data payload.
  payload: StreamDataPayload;
}

interface StreamEnd {
  // Id of the stream that has ended.
  id: number;
  // If the stream ended due to an error, the following fields will contain
  // the numeric error code as well as a message describing the error.
  errorCode?: number;
  errorMessage?: string;
}

type ServerMessage =
  | { Response: ResponseSuccess }
  | { Error: ResponseError }
  | { StreamData: StreamData }
  | { StreamEnd: StreamEnd };
```

#### **`ServerInfo`**

Represents information about the server any any setting defaults, limits, and other configurable parameters.

```rust
struct VersionInfo {
  /// Major version number.
  ///
  /// Incremented when the data format changes in backwards-incompatible ways.
  major: u16,
  /// Minor version number.
  ///
  /// Incremented when the data format changes in backwards-compatible ways.
  minor: u16,
  /// Patch version number.
  ///
  /// Incremented when changes to the protocol that do not affect the data format have occurred.
  /// Primarily informational.
  patch: u16,
}

struct BoundedValueWithDefault<T> {
  /// Minimum allowed value for this parameter.
  min: T,
  /// Maximum allowed value for this parameter.
  max: T,
  /// Default value for this parameter.
  default: T,
}

struct QuoteUpdateSettings {
  /// Bounds and defaults for the `intervalMs` field of [`QuoteUpdateParams`].
  intervalMs: BoundedValueWithDefault<u64>,
  /// Bounds and defaults for the `numQuotes` field.
  numQuotes: BoundedValueWithDefault<u32>,
}

struct SwapSettings {
  /// Default and bounds for [`SwapParams::slippage_bps`].
  slippageBps: BoundedValueWithDefault<u16>,
  /// Default value for [`SwapParams::only_direct_routes`].
  onlyDirectRoutes: bool,
  /// Default value for [`SwapParams::add_size_constraint`].
  addSizeConstraint: bool,
}

struct TransactionSettings {
  /// Default value for [`TransactionParams::close_input_token_account`].
  closeInputTokenAccount: bool,
  /// Default value for [`TransactionParams::create_output_token_account`].
  createOutputTokenAccount: bool,
}

struct ConnectionSettings {
  /// Number of concurrent streams the user is allowed.
  concurrentStreams: u32,
}

struct ServerSettings {
  /// Settings and parameter bounds for quote updates.
  quoteUpdate: QuoteUpdateSettings,
  /// Settings and parameter bounds for swaps.
  swap: SwapSettings,
  /// Settings and parameter bounds for transaction generation.
  transaction: TransactionSettings,
  /// Settings and limits for the user's connection to the server.
  connection: ConnectionSettings,
}

struct ServerInfo {
  /// Server protocol version information.
  protocolVersion: VersionInfo,
  /// Server settings and parameter bounds.
  settings: ServerSettings,
}
```

In TypeScript:

```typescript
interface VersionInfo {
  /// Major version number.
  major: number;
  /// Minor version number.
  minor: number;
  /// Patch version number.
  patch: number;
}

interface QuoteUpdateSettings {
  // Bounds and default for `intervalMs` parameter.
  intervalMs: { min: number; max: number; default: number };
  // Bounds and default for `numQuotes` parameter.
  numQuotes: { min: number; max: number; default: number };
}

interface SwapSettings {
  // Default and bounds for `slippageBps``
  slippageBps: { min: number; max: number; default: number };
  // Default value for `onlyDirectRoutes`
  onlyDirectRoutes: boolean;
  // Default value for `addSizeConstraint`
  addSizeConstraint: boolean;
}

interface TransactionSettings {
  // Default value for `closeInputTokenAccount` field for transaction params.
  closeInputTokenAccount: boolean;
  // Default value for `createOutputTokenAccount` field for transaction params.
  createOutputTokenAccount: boolean;
}

interface ConnectionSettings {
  // Number of concurrent streams the user is allowed.
  concurrentStreams: number;
}

interface ServerSettings {
  // Settings and parameter bounds for quote updates.
  quoteUpdate: QuoteUpdateSettings;
  // Settings and parameter bounds for swaps.
  swap: SwapSettings;
  // Settings and parameter bounds for transaction generation.
  transaction: TransactionSettings;
  // Settings and limits for the user's connection to the server.
  connection: ConnectionSettings;
}

interface ServerInfo {
  /// Server protocol version information.
  protocolVersion: VersionInfo;
  // Server settings and parameter bounds.
  settings: ServerSettings;
}
```

#### **`QuoteSwapStreamResponse`**

Response containing information about a new stream of swap quotes.

Currently mainly contains information about settings that may have been filled in by the server.

A response with this payload **SHOULD** also contain a `StreamStart`.

```rust
struct QuoteSwapStreamResponse {
  /// The interval, in milliseconds, in which the server will provide updates to the quotes.
  intervalMs: u64,
}
```

In TypeScript:

```typescript
interface QuoteSwapStreamResponse {
  // The interval, in milliseconds, in which the server will provide updates to the quotes.
  intervalMs: number;
}
```

#### **`StopStreamResponse`**

Response sent by the server that indicates that a stream has been successfully stopped. The client may still receive one or more `StreamData` messages for this stream if they have already been queued, followed by a `StreamEnd` message.

```rust
struct StopStreamResponse {
  // Identifier of the stream that was stopped.
  id: u32,
}
```

In TypeScript:

```typescript
interface StopStreamResponse {
  // Identifier of the stream that was stopped.
  id: number;
}
```

#### **`VenueInfo`**

Venue information sent by the server as part of a response to a `GetVenuesRequest`.

```rust
struct VenueInfo {
  // List of venue labels. Each is a valid value to use in the `dexes` and
  // `excludeDexes` parameters.
  labels: Vec<String>,
  // Program ID corresponding to each label, if requested.
  programIds: Option<Vec<Pubkey>>,
}
```

In TypeScript:

```typescript
interface VenueInfo {
  labels: string[];
  programIds?: Pubkey[];
}
```

#### **`ProviderInfo`**

Provider information sent by the server as part of a response to a `ListProvidersRequest`.

```rust
struct ProviderInfo {
  // ID of this provider.
  id: String,
  // Human-readable name for the provider.
  name: String,
  // What kind of provider this is, e.g. dex aggregator, request-for-quotes, etc.
  kind: ProviderKind,
  // URI for a 48x48 icon for the provider, if requested and available.
  iconUri48: Option<String>,
}

enum ProviderKind {
  DexAggregator,
  RFQ,
}
```

In TypeScript:

```typescript
interface ProviderInfo {
  id: string;
  name: string;
  kind: ProviderKind;
  iconUri48?: string;
}

type ProviderKind = "DexAggregator" | "RFQ";
```

### Stream Data

This section details the various types of data that may be streamed to the client by the server.

#### **`SwapQuotes`**

A set of quotes for a swap transaction, including the instructions necessary to execute the swap.

```rust
struct RoutePlanStep {
  /// Which AMM is being executed on at this step.
  ammKey: Pubkey,
  /// Label for the protocol being used.
  ///
  /// Examples: "Raydium AMM", "Phoenix", etc.
  label: String,
  /// Address of the input mint for this swap.
  inputMint: Pubkey,
  /// Address of the output mint for this swap.
  outputMint: Pubkey,
  /// How many input tokens are expected to go through this step.
  inAmount: u64,
  /// How many output tokens are expected to come out of this step.
  outAmount: u64,
  /// What what proportion, in parts per billion, of the order flow is allocated to flow through this pool.
  allocPpb: u32,
  /// Address of the mint in which the fee is charged.
  feeMint: Option<Pubkey>,
  /// The amount of tokens charged as a fee for this swap.
  feeAmount: Option<u64>,
  /// Context slot for the pool data, if known.
  contextSlot: Option<u64>,
}

struct PlatformFee {
  /// Amount of tokens taken as a fee.
  amount: u64,
  /// Fee percentage, in basis points.
  fee_bps: u8,
}

struct SwapRoute {
  /// How many input tokens are expected to go through this route.
  inAmount: u64,
  /// How many output tokens are expected to come out of this route.
  outAmount: u64,
  /// Amount of slippage incurred, in basis points.
  slippageBps: u16,
  /// Platform fee information, if such a fee is charged by the provider.
  platformFee: Option<PlatformFee>,
  /// Topologically ordered DAG containing the steps that comprise this route.
  steps: Vec<RoutePlanStep>,
  /// Instructions needed to execute the route.
  /// May not be provided if a full transaction is provided instead.
  instructions: Vec<Instruction>,
  /// Address lookup tables necessary to load.
  addressLookupTables: Vec<Pubkey>,
  /// Context slot for the route provided.
  contextSlot: Option<u64>,
  /// Amount of time taken to generate the quote in nanoseconds, if known.
  timeTakenNs: Option<u64>,
  /// If this route expires, the time at which it expires, as a millisecond UNIX
  /// timestamp.
  expiresAtMs: Option<u64>,
  /// If this route expires by slot, the last slot at which the route is valid.
  expiresAfterSlot: Option<u64>,
  /// The number of compute units this transaction is expected to consume, if known.
  computeUnits: Option<u64>,
  /// Recommended number of compute units to use for the budget for this route, if known.
  /// The number of compute units used by a route can fluctuate based on changes on-chain,
  /// so the server will recommend a higher limit that should allow the transaction to execute
  /// in the vast majority of cases.
  computeUnitsSafe: Option<u64>,
  /// Transaction for the user to sign, if instructions are not provided.
  transaction: Option<Vec<u8>>,
  /// Provider-specific reference ID for this quote.
  ///
  /// Mainly provided by RFQ-based providers such as Pyth Express Relay and Hashflow.
  reference_id: Option<String>,
}

struct SwapQuotes {
  /// Unique identifier for the quote.
  id: String,
  /// Address of the input mint for this quote.
  inputMint: Pubkey,
  /// Address of the output mint for this quote.
  outputMint: Pubkey,
  /// What swap mode was used for the quotes.
  swapMode: SwapMode,
  /// Amount used for the quotes.
  amount: u64,
  /// A mapping of a provider identifier to their quoted route.
  quotes: HashMap<String, SwapRoute>,
}
```

In TypeScript:

```typescript
interface RoutePlanStep {
  // Which AMM is being executed on at this step.
  ammKey: Uint8Array;
  // Label for the protocol being used.
  //
  // Examples: "Raydium AMM", "Phoenix", etc.
  label: string;
  // Address of the input mint for this swap.
  inputMint: Uint8Array;
  // Address of the output mint for this swap.
  outputMint: Uint8Array;
  // How many input tokens are expected to go through this step.
  inAmount: number;
  // How many output tokens are expected to come out of this step.
  outAmount: number;
  // What what proportion, in parts per billion, of the order flow is allocated
  // to flow through this pool.
  allocPpb: number;
  // Address of the mint in which the fee is charged.
  feeMint?: Uint8Array;
  // The amount of tokens charged as a fee for this swap.
  feeAmount?: number;
  // Context slot for the pool data, if known.
  contextSlot?: number;
}

interface PlatformFee {
  /// Amount of tokens taken as a fee.
  amount: number;
  /// Fee percentage, in basis points.
  fee_bps: number;
}

interface SwapRoute {
  // How many input tokens are expected to go through this route.
  inAmount: number;
  // How many output tokens are expected to come out of this route.
  outAmount: number;
  // Amount of slippage encurred, in basis points.
  slippageBps: number;
  // Platform fee information; if such a fee is charged by the provider.
  platformFee?: PlatformFee;
  // Topologically ordered DAG containing the steps that comprise this route.
  steps: RoutePlanStep[];
  // Instructions needed to execute the route.
  instructions: Instruction[];
  // Address lookup tables necessary to load.
  addressLookupTables: Pubkey[];
  // Context slot for the route provided.
  contextSlot?: number;
  // Amount of time taken to generate the quote in nanoseconds; if known.
  timeTaken?: number;
  // If this route expires by time, the time at which it expires,
  // as a millisecond UNIX timestamp.
  expiresAtMs?: number;
  // If this route expires by slot, the last slot at which the route is valid.
  expiresAfterSlot?: number;
  // The number of compute units this transaction is expected to consume, if known.
  computeUnits?: number;
  // Recommended number of compute units to use for the budget for this route, if known.
  // The number of compute units used by a route can fluctuate based on changes on-chain,
  // so the server will recommend a higher limit that should allow the transaction to execute
  // in the vast majority of cases.
  computeUnitsSafe?: number;
  // Transaction for the user to sign, if instructions not provided.
  transaction?: Uint8Array;
  // Provider-specific reference ID for this quote.
  //
  // Mainly provided by RFQ-based providers such as Pyth Express Relay and Hashflow.
  referenceId?: string;
}

interface SwapQuotes {
  // Unique Quote identifier.
  id: string;
  // Address of the input mint for this quote.
  inputMint: Uint8Array;
  // Address of the output mint for this quote.
  outputMint: Uint8Array;
  // What swap mode was used for the quotes.
  swapMode: SwapMode;
  // Amount used for the quotes.
  amount: number;
  // A mapping of a provider identifier to their quoted route.
  quotes: { [key: string]: SwapRoute };
}
```
