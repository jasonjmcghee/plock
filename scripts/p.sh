#!/usr/bin/env bash
# From https://x.com/rauchg/status/1748961908370321474?s=20
jq -n \
		--arg content "$*" \
		'{
      "model": "pplx-7b-online",
      "messages": [
        {
          "role": "system",
          "content": "Be precise and concise."
        },
        {
          "role": "user",
          "content": ($content | tojson)
        }
      ],
      "stream": true
    }' | curl --silent \
		--request POST \
		--url https://api.perplexity.ai/chat/completions \
		--header 'accept: application/json' \
		--header "authorization: Bearer $PERPLEXITY_API" \
		--header 'content-type: application/json' \
		--data @- |
		while read -r line; do
			echo "${line:6}" | jq -j '.choices[0].delta.content'
		done && echo ""