# swap

> Request for a base64-encoded unsigned swap transaction based on the `/quote` response


## OpenAPI

````yaml openapi-spec/swap/swap.yaml post /swap
paths:
  path: /swap
  method: post
  servers:
    - url: https://lite-api.jup.ag/swap/v1
      description: Free tier API endpoint with rate limits
    - url: https://api.jup.ag/swap/v1
      description: >-
        Paid tier API endpoint with higher rate limits to be used with an API
        Key
    - url: https://preprod-quote-api.jup.ag/
      description: This is a staging endpoint for tests
  request:
    security: []
    parameters:
      path: {}
      query: {}
      header: {}
      cookie: {}
    body:
      application/json:
        schemaArray:
          - type: object
            properties:
              userPublicKey:
                allOf:
                  - type: string
              payer:
                allOf:
                  - description: >
                      - Allow a custom payer to pay for the transaction fees and
                      rent of token accounts

                      - Note that users can close their ATAs elsewhere and have
                      you reopen them again, your fees should account for this
                    type: string
              wrapAndUnwrapSol:
                allOf:
                  - description: >
                      - To automatically wrap/unwrap SOL in the transaction, as
                      WSOL is an SPL token while native SOL is not

                      - When true and input mint is SOL, it will wrap the SOL
                      amount to WSOL and swap

                      - When true and output mint is SOL, it will unwrap the
                      WSOL back to SOL

                      - When false and input mint is SOL, it will use existing
                      WSOL amount to swap

                      - When false and output mint is SOL, it will not unwrap
                      the WSOL to SOL

                      - To set this parameter to false, you need to have the
                      WSOL token account initialized
                    type: boolean
                    default: true
              useSharedAccounts:
                allOf:
                  - description: >
                      - The default is determined dynamically by the routing
                      engine, allowing us to optimize for compute units, etc

                      - This enables the usage of shared program accounts, this
                      is essential as complex routing will require multiple
                      intermediate token accounts which the user might not have

                      - If true, you do not need to handle the creation of
                      intermediate token accounts for the user

                      - Do note, shared accounts route will fail on some new
                      AMMs (low liquidity token)
                    type: boolean
              feeAccount:
                allOf:
                  - description: >
                      - An initialized token account that will be used to
                      collect fees

                      - The mint of the token account **can only be either the
                      input or output mint of the swap**

                      - Swap API no longer requires the use of the Referral
                      Program

                      - If `platformFeeBps` is passed in `/quote`, the
                      `feeAccount` must be passed as well
                    type: string
              trackingAccount:
                allOf:
                  - description: >
                      - Specify any public key that belongs to you to track the
                      transactions

                      - Useful for integrators to get all the swap transactions
                      from this public key

                      - Query the data using a block explorer like
                      Solscan/SolanaFM or query like Dune/Flipside
                    type: string
              prioritizationFeeLamports:
                allOf:
                  - description: >
                      - To specify a level or amount of additional fees to
                      prioritize the transaction

                      - It can be used for EITHER priority fee OR Jito tip (not
                      both at the same time)

                      - If you want to include both, you will need to use
                      `/swap-instructions` to add both at the same time

                      - Defaults to `auto`, but preferred to use
                      `priorityLevelWithMaxLamports` as it may be more accurate
                      when accounting local fee market

                      - Fixed lamports can be passed in as an integer in the
                      `prioritizationFeeLamports` parameter
                    oneOf:
                      - $ref: '#/components/schemas/PriorityLevelWithMaxLamports'
                      - $ref: '#/components/schemas/JitoTipLamports'
                      - $ref: '#/components/schemas/JitoTipLamportsWithPayer'
              asLegacyTransaction:
                allOf:
                  - description: >
                      - Builds a legacy transaction rather than the default
                      versioned transaction

                      - Used together with `asLegacyTransaction` in `/quote`,
                      otherwise the transaction might be too large
                    type: boolean
                    default: false
              destinationTokenAccount:
                allOf:
                  - description: >
                      - Public key of a token account that will be used to
                      receive the token out of the swap

                      - If not provided, the signer's token account will be used

                      - If provided, we assume that the token account is already
                      initialized

                      - `destinationTokenAccount` and `nativeDestinationAccount`
                      are mutually exclusive
                    type: string
              nativeDestinationAccount:
                allOf:
                  - description: >
                      - Public key of an account that will be used to receive
                      the native SOL token out of the swap

                      - If not provided, the swap will default unwrap the WSOL
                      and transfer the native SOL to the swap authority account

                      - If provided, we will unwrap the WSOL and transfer the
                      native SOL to the account

                      - Only works if the output mint is SOL, is using the V2
                      instructions and the account passed in is not owned by
                      token program

                      - When sending native SOL to a new account, you must swap
                      at least enough to cover the rent required to create it.

                      - `destinationTokenAccount` and `nativeDestinationAccount`
                      are mutually exclusive
                    type: string
              dynamicComputeUnitLimit:
                allOf:
                  - description: >
                      - When enabled, it will do a swap simulation to get the
                      compute unit used and set it in ComputeBudget's compute
                      unit limit

                      - This incurs one extra RPC call to simulate this

                      - We recommend to enable this to estimate compute unit
                      correctly and reduce priority fees needed or have higher
                      chance to be included in a block
                    type: boolean
                    default: false
              skipUserAccountsRpcCalls:
                allOf:
                  - description: >
                      - When enabled, it will not do any additional RPC calls to
                      check on required accounts

                      - The returned swap transaction will still attempt to
                      create required accounts regardless if it exists or not
                    type: boolean
                    default: false
              dynamicSlippage:
                allOf:
                  - description: >
                      - When enabled, it estimates slippage and apply it in the
                      swap transaction directly, overwriting the `slippageBps`
                      parameter in the quote response.

                      - This is no longer maintained, we are focusing efforts on
                      RTSE on Ultra Swap API
                    type: boolean
                    default: false
              computeUnitPriceMicroLamports:
                allOf:
                  - description: >
                      - To use an exact compute unit price to calculate priority
                      fee

                      - `computeUnitLimit (1400000) *
                      computeUnitPriceMicroLamports`

                      - We recommend using `prioritizationFeeLamports` and
                      `dynamicComputeUnitLimit` instead of passing in your own
                      compute unit price
                    type: integer
                    format: uint64
              blockhashSlotsToExpiry:
                allOf:
                  - description: >
                      - Pass in the number of slots we want the transaction to
                      be valid for

                      - Example: If you pass in 10 slots, the transaction will
                      be valid for ~400ms * 10 = approximately 4 seconds before
                      it expires
                    type: integer
                    format: uint8
              quoteResponse:
                allOf:
                  - $ref: '#/components/schemas/QuoteResponse'
            required: true
            refIdentifier: '#/components/schemas/SwapRequest'
            requiredProperties:
              - userPublicKey
              - quoteResponse
            example:
              userPublicKey: jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3
              quoteResponse:
                inputMint: So11111111111111111111111111111111111111112
                inAmount: '1000000'
                outputMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
                outAmount: '125630'
                otherAmountThreshold: '125002'
                swapMode: ExactIn
                slippageBps: 50
                platformFee: null
                priceImpactPct: '0'
                routePlan:
                  - swapInfo:
                      ammKey: AvBSC1KmFNceHpD6jyyXBV6gMXFxZ8BJJ3HVUN8kCurJ
                      label: Obric V2
                      inputMint: So11111111111111111111111111111111111111112
                      outputMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
                      inAmount: '1000000'
                      outAmount: '125630'
                      feeAmount: '5'
                      feeMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
                    percent: 100
              prioritizationFeeLamports:
                priorityLevelWithMaxLamports:
                  maxLamports: 10000000
                  priorityLevel: veryHigh
              dynamicComputeUnitLimit: true
        examples:
          example:
            value:
              userPublicKey: jdocuPgEAjMfihABsPgKEvYtsmMzjUHeq9LX4Hvs7f3
              quoteResponse:
                inputMint: So11111111111111111111111111111111111111112
                inAmount: '1000000'
                outputMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
                outAmount: '125630'
                otherAmountThreshold: '125002'
                swapMode: ExactIn
                slippageBps: 50
                platformFee: null
                priceImpactPct: '0'
                routePlan:
                  - swapInfo:
                      ammKey: AvBSC1KmFNceHpD6jyyXBV6gMXFxZ8BJJ3HVUN8kCurJ
                      label: Obric V2
                      inputMint: So11111111111111111111111111111111111111112
                      outputMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
                      inAmount: '1000000'
                      outAmount: '125630'
                      feeAmount: '5'
                      feeMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
                    percent: 100
              prioritizationFeeLamports:
                priorityLevelWithMaxLamports:
                  maxLamports: 10000000
                  priorityLevel: veryHigh
              dynamicComputeUnitLimit: true
  response:
    '200':
      application/json:
        schemaArray:
          - type: object
            properties:
              swapTransaction:
                allOf:
                  - type: string
              lastValidBlockHeight:
                allOf:
                  - type: integer
                    format: uint64
              prioritizationFeeLamports:
                allOf:
                  - type: integer
                    format: uint64
            refIdentifier: '#/components/schemas/SwapResponse'
            requiredProperties:
              - swapTransaction
              - lastValidBlockHeight
        examples:
          example:
            value:
              swapTransaction: <string>
              lastValidBlockHeight: 123
              prioritizationFeeLamports: 123
        description: Successful response
  deprecated: false
  type: path
components:
  schemas:
    QuoteResponse:
      type: object
      required:
        - inputMint
        - outputMint
        - inAmount
        - outAmount
        - otherAmountThreshold
        - swapMode
        - slippageBps
        - priceImpactPct
        - routePlan
      properties:
        inputMint:
          type: string
        inAmount:
          type: string
        outputMint:
          type: string
        outAmount:
          type: string
          description: |
            - Calculated output amount from routing engine
            - The value includes platform fees and DEX fees, excluding slippage
        otherAmountThreshold:
          type: string
          description: >
            - Calculated minimum output amount after accounting for
            `slippageBps` on the `outAmount` value

            - Not used by `/swap` endpoint to build transaction
        swapMode:
          $ref: '#/components/schemas/SwapMode'
          required: true
        slippageBps:
          type: integer
          format: uint16
          minimum: 0
        platformFee:
          $ref: '#/components/schemas/PlatformFee'
        priceImpactPct:
          type: string
        routePlan:
          type: array
          items:
            $ref: '#/components/schemas/RoutePlanStep'
        contextSlot:
          type: integer
          format: uint64
        timeTaken:
          type: number
    SwapMode:
      type: string
      enum:
        - ExactIn
        - ExactOut
    PlatformFee:
      type: object
      properties:
        amount:
          type: string
        feeBps:
          type: integer
          format: uint16
    RoutePlanStep:
      type: object
      properties:
        swapInfo:
          $ref: '#/components/schemas/SwapInfo'
        percent:
          type: integer
          format: uint8
        bps:
          type: integer
          format: uint16
      required:
        - swapInfo
    SwapInfo:
      type: object
      required:
        - ammKey
        - inputMint
        - outputMint
        - inAmount
        - outAmount
        - feeAmount
        - feeMint
      properties:
        ammKey:
          type: string
        label:
          type: string
        inputMint:
          type: string
        outputMint:
          type: string
        inAmount:
          type: string
        outAmount:
          type: string
        feeAmount:
          type: string
        feeMint:
          type: string
    PriorityLevelWithMaxLamports:
      title: priorityLevelWithMaxLamports
      type: object
      properties:
        priorityLevelWithMaxLamports:
          type: object
          properties:
            priorityLevel:
              type: string
              enum:
                - medium
                - high
                - veryHigh
            maxLamports:
              type: integer
              format: uint64
              description: >
                - Maximum lamports to cap the priority fee estimation, to
                prevent overpaying
            global:
              type: boolean
              default: false
              description: >
                - A boolean to choose between using a global or local fee market
                to estimate. If `global` is set to `false`, the estimation
                focuses on fees relevant to the **writable accounts** involved
                in the instruction.
          required:
            - priorityLevel
            - maxLamports
          additionalProperties: false
      required:
        - priorityLevelWithMaxLamports
      additionalProperties: false
    JitoTipLamports:
      title: jitoTipLamports
      type: object
      properties:
        jitoTipLamports:
          type: integer
          format: uint64
          description: >
            - Exact amount of tip to use in a tip instruction

            - Refer to Jito docs on how to estimate the tip amount based on
            percentiles

            - It has to be used together with a connection to a Jito RPC

            - See their docs at https://docs.jito.wtf/
      required:
        - jitoTipLamports
      additionalProperties: false
    JitoTipLamportsWithPayer:
      title: jitoTipLamportsWithPayer
      type: object
      properties:
        jitoTipLamportsWithPayer:
          type: object
          properties:
            lamports:
              type: integer
              format: uint64
              description: Exact amount of lamports to use for the tip
            payer:
              type: string
              description: Public key of an account that will be used to pay for the tip
          required:
            - lamports
            - payer
          additionalProperties: false
      required:
        - jitoTipLamportsWithPayer
      additionalProperties: false

````