# RSON

rson is a **fast** and **concurrent** Slack notification router built on top of **hyper**.

It makes adding apps and integrations on the fly trivial, providing a single URL for Slack to send POST requests to. rson then takes care of parsing incoming requests and determining where they should be routed to, be that Google App Scripts, 3rd Party Software, or other custom APIs. 

Getting started is as easy as:

```
    let addr = "localhost:1337".parse().unwrap();
    let mapfile = Path::new("mappings.json");
    let slack_response = "{\"text\": \" ✅ Your request has been received!\"}";
    match start_server(addr, mapfile, slack_response) {
        Ok(_) => {}
        Err(error) => panic!(error),
    }
```

## Mappings

The `mappings.json` file is used to map a `callback_id` to a `URL`, which the server then uses to forward the Slack POST request to. An example of the structure is as follows:

```
{
  "some_id": "https://script.google.com/macros/s/****/exec",
  "nyan" : "http://www.nyan.cat/",
  "debug" : "http://httpbin.org/post"
  ...etc
}

```
