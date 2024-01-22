#!/usr/bin/env bash
# From https://x.com/rauchg/status/1748961908370321474?s=20
jq -n \
		--arg content "$*" \
		'{
      "model": "gpt-3.5-turbo-1106",
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
		--url https://api.openai.com/v1/chat/completions \
		--header 'accept: application/json' \
		--header "authorization: Bearer $OPENAI_API" \
		--header 'content-type: application/json' \
		--data @- |
		while read -r line; do
		  partial="$(echo "${line:6}")"
		  if [[ "$partial" == "[DONE]" ]]; then
		    break
		  fi
      printf "$(echo "${partial//$'\n'/\\n}" | jq '.choices[0].delta.content' | awk -F'"' '{print $2}')"
		done && echo ""