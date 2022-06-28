# wasi-cloud
wasi-cloud is a straight-forward attempt to see if ~80% of the cloud native APIs can be modeled in Wasm component interfaces compiled to the WebAssembly System Interface or WASI specification. If so, the developer experience could take a step _backward into tight productivity_, with quick development feedback loops while removing the domain specific cloud knowledge currently required. Enabling developers to build apps and ops teams to deploy them with radically separated fields of expertise.  

Fundamentally, this experiment tests out the the possibility that system interface-based webassemly components (using wit) could drive a useful separation between the app developer, deployment environment, and operator to enable critical optimizations and experiences that can then be built to optimize both, making distributed, cloud-native applications easier for both dev and ops as well as extending the reach of what we currently call "cloud native" into constrained, heterogeneous environments that are not called "hyperscale public cloud". 

The approach taken here does not involve any new technologies; it tries to lay down a test case to determine whether 80% of all cloud-native apps could be built more easily and operated more easily by more people. Note that this means that we are not assuming that we can reach the other 20% right away, if ever. Instead, it embraces the concept that Visual Basic enabled vastly easier application building because it made that first 80% so easy to do that the power of every developer was exponentially increased. Increasing the delivery velocity of the 80% was fundamental to the explosion of Windows in the first place, and if there's one thing that is true of the current environment it's that everyone agrees it's hard to know where to start. 

## Preamble to the experiment

Getting to the essence of Visual Basic:

```
Visual Basic 6 accomplished its goals by abstracting away the complexity of the underlying Windows OS. Simple things were very simple to accomplish. On the other hand, complex things, such as dealing with threads, were impossible. My rule of thumb for Visual Basic 6 was: if I couldn’t do it within 10 minutes, I couldn’t do it at all.
```

  -- https://docs.microsoft.com/en-us/archive/msdn-magazine/2012/june/don-t-get-me-started-the-silent-majority-why-visual-basic-6-still-thrives

this we cant do in cloud native.

In cloud native, we've achieved the explosive growth of the public cloud (helped by the epidemic), but the acceleration curve (though still healthy) is already flattening. Evidence:
- the requirement of all three hyperscale cloud competitors to begin building private cloud attach in the form of Arc-like agents.
- the rise of [CDN clouds](https://stratechery.com/2021/cloudflares-disruption/) out of the original CDN function platforms like Fastly and CloudFlare, and the rise of the [attack on public cloud data egress charges](https://blog.cloudflare.com/aws-egregious-egress/).
- the focus by [prominent VCs](https://a16z.com/2021/05/27/cost-of-cloud-paradox-market-cap-cloud-lifecycle-scale-growth-repatriation-optimization/) and [startups](https://www.fermyon.com/blog/dont-repatriate-servers) on the dramatic reduction of costs by limiting or "repatriating" your compute off of hyperscale

By far most future compute will appear in lower-powered, smaller chips that will not be the stock in trade of hyperscale for some time, instead adding compute power to the "edge" -- where compute will be more heterogeneous and more constrained than the infinite, homogeneous scale out of hyperscale. 

That last is the future explosive growth of cloud over the next five to ten years, and it includes places we know about but the outside world only vaguely. Space is the edge: constrained, heterogeneous, with satelite and orbital data centers coming within the decade. Industrial automation is the edge: assembly lines are constrained, heterogeneous, long using native code systems that constrain democratic labor supply and have not seen the explosion of productivity gained by cloud-native operational practices. Vehicles are moving, small data centers, as are ships, planes, buses, trains, cell towers, billboards, and so on. 

Distributed application development methods must become simpler in order to scale out developer impact and scope; we know it's too hard for most people to choose not only stacks, but _combinations of composed stacks_ in order to build what they need, and of course select clouds that have proprietary but public APIs to consume. It's all so hard that bigger companies now build [custom developer experience platforms](https://redmonk.com/sogrady/2020/10/06/developer-experience-gap/) _on_ public clouds instead of using those clouds' platforms. 

wasi-cloud is a strait-forward attempt to see if ~80% of the cloud native APIs can be modeled in Wasm component interfaces compiled to the WebAssembly System Interface or WASI specification. If so, the developer experience could take a step _backward into tight productivity_, with quick development feedback loops while removing the domain specific cloud knowledge currently required. Enabling developers to build apps and ops teams to deploy them with radically separated fields of expertise.

## Structure
- `/wit`: the wasi-cloud specification written in `*.wit` format (see [WIT](https://github.com/bytecodealliance/wit-bindgen/blob/main/WIT.md))
- `/src`: the wasi-cloud host cli 
- `/crates`: host implementation
- `/examples`: guest examples
- `/tests`: guest tests

## Build
- Run `make build`

## Run
- Run `make run`
