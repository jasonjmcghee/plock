#!/usr/bin/env bash

jq -n \
    --arg content "$*" \
    '{
       "model": "dall-e-2",
       "prompt": $content,
       "n": 1,
       "size": "1024x1024",
       "response_format": "b64_json"
     }' | curl \
    --request POST \
    --url https://api.openai.com/v1/images/generations \
    --header 'accept: application/json' \
    --header "authorization: Bearer $OPENAI_API" \
    --header 'content-type: application/json' \
    --data @- | jq -r '.data[0].b64_json'