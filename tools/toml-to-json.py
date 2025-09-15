import json
import sys
import tomllib

with open(sys.argv[1], "rb") as f:
    json.dump(tomllib.load(f), sys.stdout, ensure_ascii=False, indent=2, sort_keys=True)
