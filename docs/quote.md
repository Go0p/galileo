# quote

> Request for a quote to be used in `POST /swap`


## OpenAPI

````yaml openapi-spec/swap/swap.yaml get /quote
paths:
  path: /quote
  method: get
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
      query:
        inputMint:
          schema:
            - type: string
              required: true
        outputMint:
          schema:
            - type: string
              required: true
        amount:
          schema:
            - type: integer
              required: true
              description: |
                - Raw amount to swap (before decimals)
                - Input Amount if `SwapMode=ExactIn`
                - Output Amount if `SwapMode=ExactOut`
        slippageBps:
          schema:
            - type: integer
              description: >
                - Default: 50

                - This is threshold denoted in basis points.

                - If exact in and output amount exceeds the threshold, then the
                swap transaction will fail.
              default: 50
        swapMode:
          schema:
            - type: enum<string>
              enum:
                - ExactIn
                - ExactOut
              description: >
                - ExactOut is for supporting use cases where you need an exact
                output amount

                - In the case of `ExactIn`, the slippage is on the output token

                - In the case of `ExactOut`, the slippage is on the input token

                - Not all AMMs support `ExactOut`: Currently only Orca
                Whirlpool, Raydium CLMM, Raydium CPMM

                - We do not recommend using `ExactOut` for most use cases
              default: ExactIn
        dexes:
          schema:
            - type: array
              items:
                allOf:
                  - type: string
              description: >
                - Multiple DEXes can be pass in by comma separating them

                - For example: `dexes=Raydium,Orca+V2,Meteora+DLMM`

                - If a DEX is indicated, the route will **only use** that DEX

                - Full list of DEXes here:
                https://lite-api.jup.ag/swap/v1/program-id-to-label
        excludeDexes:
          schema:
            - type: array
              items:
                allOf:
                  - type: string
              description: >
                - Multiple DEXes can be pass in by comma separating them

                - For example: `excludeDexes=Raydium,Orca+V2,Meteora+DLMM`

                - If a DEX is indicated, the route will **not use** that DEX

                - Full list of DEXes here:
                https://lite-api.jup.ag/swap/v1/program-id-to-label
        restrictIntermediateTokens:
          schema:
            - type: boolean
              description: >
                - Restrict intermediate tokens within a route to a set of more
                stable tokens

                - This will help to reduce exposure to potential high slippage
                routes
              default: true
        onlyDirectRoutes:
          schema:
            - type: boolean
              description: |
                - Direct route limits Jupiter routing to single hop routes only
                - This may result in worse routes
              default: false
        asLegacyTransaction:
          schema:
            - type: boolean
              description: >
                - Instead of using versioned transaction, this will use the
                legacy transaction
              default: false
        platformFeeBps:
          schema:
            - type: integer
              description: >
                - Take fees in basis points

                - If `platformFeeBps` is passed in, the `feeAccount` in `/swap`
                must be passed as well
        maxAccounts:
          schema:
            - type: integer
              description: >
                - Rough estimate of the max accounts to be used for the quote

                - Useful if composing your own transaction or to be more precise
                in resource accounting for better routes
              default: 64
        instructionVersion:
          schema:
            - type: enum<string>
              enum:
                - V1
                - V2
              description: |
                - The version of instruction to use in the swap program
              default: V1
        dynamicSlippage:
          schema:
            - type: boolean
              description: >
                - No longer applicable, only required to pass in via `/swap`
                endpoint
              default: false
      header: {}
      cookie: {}
    body: {}
  response:
    '200':
      application/json:
        schemaArray:
          - type: object
            properties:
              inputMint:
                allOf:
                  - type: string
              inAmount:
                allOf:
                  - type: string
              outputMint:
                allOf:
                  - type: string
              outAmount:
                allOf:
                  - type: string
                    description: >
                      - Calculated output amount from routing engine

                      - The value includes platform fees and DEX fees, excluding
                      slippage
              otherAmountThreshold:
                allOf:
                  - type: string
                    description: >
                      - Calculated minimum output amount after accounting for
                      `slippageBps` on the `outAmount` value

                      - Not used by `/swap` endpoint to build transaction
              swapMode:
                allOf:
                  - $ref: '#/components/schemas/SwapMode'
                    required: true
              slippageBps:
                allOf:
                  - type: integer
                    format: uint16
                    minimum: 0
              platformFee:
                allOf:
                  - $ref: '#/components/schemas/PlatformFee'
              priceImpactPct:
                allOf:
                  - type: string
              routePlan:
                allOf:
                  - type: array
                    items:
                      $ref: '#/components/schemas/RoutePlanStep'
              contextSlot:
                allOf:
                  - type: integer
                    format: uint64
              timeTaken:
                allOf:
                  - type: number
            refIdentifier: '#/components/schemas/QuoteResponse'
            requiredProperties:
              - inputMint
              - outputMint
              - inAmount
              - outAmount
              - otherAmountThreshold
              - swapMode
              - slippageBps
              - priceImpactPct
              - routePlan
        examples:
          example:
            value:
              inputMint: <string>
              inAmount: <string>
              outputMint: <string>
              outAmount: <string>
              otherAmountThreshold: <string>
              swapMode: ExactIn
              slippageBps: 1
              platformFee:
                amount: <string>
                feeBps: 123
              priceImpactPct: <string>
              routePlan:
                - swapInfo:
                    ammKey: <string>
                    label: <string>
                    inputMint: <string>
                    outputMint: <string>
                    inAmount: <string>
                    outAmount: <string>
                    feeAmount: <string>
                    feeMint: <string>
                  percent: 123
                  bps: 123
              contextSlot: 123
              timeTaken: 123
        description: Successful response to be used in `/swap`
  deprecated: false
  type: path
components:
  schemas:
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

````