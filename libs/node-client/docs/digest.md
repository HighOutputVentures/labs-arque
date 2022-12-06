```mermaid
flowchart TB
  A(Start) --> C{has next event?}
  C --> |yes| D[update state]
  D --> E[update version]
  E --> C
  C --> |no| H(End)
```
