# Project Roadmap

- [x] Configuration
  - [ ] More precise buffering configuration (e.g. for maximum decompressed payload size)
  - [ ] Load from arbitrary place (using the NEOGROK_CFG_PATH environment variable)
  - [ ] Load from many places

- [x] Implement protocols
  - [ ] HTTP
  - [x] TCP
  - [ ] UDP
    - [ ] Peer to peer

- [ ] Implement forward packet payload compression
  - [x] Deflate (done using libdeflate)
  - [x] ZStandard (done using zstd-sys)

- [ ] Implement GUI for the client
  - dioxuslabs.com or iced.rs?

- [ ] Implement web dashboard
  - dioxuslabs.com or yew.rs?

- [ ] Implement server statistics collection
- [ ] Implement database related functionality such as authorization
