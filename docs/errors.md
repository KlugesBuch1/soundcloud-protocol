# Errors

| Status | Body | What it proved | Measurement |
| --- | --- | --- | --- |
| 400 | JSON with an `errors` array | A parameter was missing | Comments without `threaded` |
| 401 | Empty | An auth gate was reached | Missing client id on two paths; `/me` and writes without a user token |
| 404 | `{}` | The object was not found | Missing track and missing user |
| 404 | Empty | The path was not found | Unknown path with a valid client id |
| 429 | Not observed | No body shape was measured | A burst of 50 track requests did not trigger it |

The 401 responses for a valid path and for `/this/path/does/not/exist` are identical. The client id is checked before the path. A 401 does not prove that a route exists.

The two 404 shapes do not carry enough information to separate a dead path from a missing object on their own. An empty body and `{}` are both possible.

The 400 body was:

```json
{"errors":["Did not get or could not parse multiple missing params"]}
```

The API sent no rate limit headers. The 429 body was not measured, so this catalogue does not describe it. No 403 HTML challenge appeared in the measured requests.

## Examples

- [error-404.json](../examples/error-404.json)
- [error-400.json](../examples/error-400.json)
