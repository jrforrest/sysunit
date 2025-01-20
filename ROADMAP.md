### 0.5

- [x] Don't show sh -x output without --debug
    - Don't set the -x flag when the running under --debug
    - Probably need to pass something down through EngineOptions for this.
- [x] UnitFile
    - Allow multiple units in a file to enable make-like workflows

### 0.6


- [x] Allow running units through things other than /bin/sh
    - Protocols on target hosts can be mapped to adapters
    - Allow an adapter mapping to be provided in CLI args or in a TOML file
      that lets the user specify the adapter to use for a given protocol.

### 0.7


- [ ] Add unit aliases
  - An alias allows a unit (along with its arguments) to be referred to by a nicer label.
    this is primarily to allow rendering of units with long argument lists in the UI in
    a way that is meaningful from the unit that depends on it.
- Syntax Ergonomics
    - [ ] Update deps syntax so it doesn't conflict with shell chars
      - Right now the captures syntax makes quotes necessary around deps
    - [ ] Allow string values in deps and emits to be unquoted
      - Shell quoting rules make double quoting necessary for these values right
        now.
- [ ] CLI Improvements
    - Show less states in default mode.  One line for loading a unit should be fine.

### 0.8

- [ ] Unit Meta
    - [ ] Unit Descriptions
      - These need to be rendered to the UI in a nice way.  Maybe allow a 'show' operation
        that just prints out the unit.
    - Allow description on unit, and a nice way to render it in the UI
    - Allow version specifiers 
      - A bunch of this logic was there previously but stripped out, should be in git history
