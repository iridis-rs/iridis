# Runtime

Once a layout and the flows are created, you can create a Runtime that will run the application. Upon creating a runtime, you can load your plugins that will be used to alter the behavior of the runtime. For example, you can load a `FileExtPlugin` that will allow you to load files with a specific extension, or a `UrlSchemePlugin` that will allow you to load files with a specific URL scheme.

```rust
let runtime = Runtime::new(
        async |file_ext: &mut FileExtLoader, url_scheme: &mut UrlSchemeLoader| Ok(()),
    )
    .await?;
```

Then you can call the `run` method to run the application. The `run` method takes a `Flows` object and a closure that will be called to load the nodes. The closure takes a `Loader` object that can be used to load the nodes.

```rust
runtime.run(flows, async move |loader: &mut Loader| {
    loader
        .load::<Transport>(operator, serde_yml::from_str("")?)
        .await?;

    loader
        .load_url(
            Url::parse("file:///path/to/some/dylib")?,
            sink,
            serde_yml::from_str("")?,
        )
        .await?;

    Ok(())
})
.await
```

The pipeline of the node loading by `url` is as follow:

- The Loader extracts the scheme of the `url`.
- It selects the plugin that matches the scheme.
- It then calls the `load` method of the plugin, passing the `url` and the `node` to load, it also provides all the `file-ext` plugins so that the `url-scheme` plugin can rely on those plugins.
