```mermaid
flowchart TB
  A(Start) --> B[retrieve new events]
  B --> C{has next event?}
  C --> |yes| D[update state]
  D --> E[update version]
  E --> C
  C --> |no| H[Stop]
```
