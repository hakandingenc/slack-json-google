var searchIndex = {};
searchIndex["rson"] = {"doc":"rson is a fast and concurrent Slack notification router built on top of hyper.","items":[[4,"Command","rson","This enum represents a command to be run on the HostMap struct.",null,null],[13,"Add","","This variant represents the command to add a new mapping to the HostMap struct or to update an existing one. The first field is the callback id, and the second field is the corresponding url.",0,null],[13,"Remove","","This variant represents the command to remove an existing mapping from the HostMap struct. Its only field is the callback id to be removed.",0,null],[13,"List","","This variant represents the command to list (by pretty printing) all mappings found in the HostMap struct.",0,null],[5,"start_server","","Creates tokio Core, initializes server, and spawns 3 threads that loop over reading from `stdin()`, resolving callback ids, and running HostMap commands.",null,{"inputs":[{"name":"socketaddr"},{"name":"path"},{"name":"str"}],"output":{"name":"result"}}],[0,"hostmap","","Callback id to url mappings",null,null],[3,"HostMap","rson::hostmap","Callback id to url mappings",null,null],[11,"default","","",1,{"inputs":[],"output":{"name":"hostmap"}}],[11,"eq","","",1,{"inputs":[{"name":"self"},{"name":"hostmap"}],"output":{"name":"bool"}}],[11,"ne","","",1,{"inputs":[{"name":"self"},{"name":"hostmap"}],"output":{"name":"bool"}}],[11,"clone","","",1,{"inputs":[{"name":"self"}],"output":{"name":"hostmap"}}],[11,"new_from_file","","Creates a new HostMap from a JSON encoded file",1,{"inputs":[{"name":"path"}],"output":{"name":"result"}}],[11,"resolve_callback","","Given a callback id, returns `Some` containing the corresponding url, or `None` if one doesn't exist",1,{"inputs":[{"name":"self"},{"name":"str"}],"output":{"generics":["string"],"name":"option"}}],[11,"insert","","Given a callback id and a url, inserts a new mapping to HostMap, or updates if the callback id already exists",1,{"inputs":[{"name":"self"},{"name":"string"},{"name":"string"}],"output":null}],[11,"remove","","Given a callback id, removes it from the HostMap, returning `Some` containing the corresponding url, or `None` if the callback id doesn't exits",1,{"inputs":[{"name":"self"},{"name":"str"}],"output":{"generics":["string"],"name":"option"}}],[11,"fmt","","",1,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[0,"server","rson","Main server module",null,null],[3,"Server","rson::server","Server struct",null,null],[11,"new","","Given required objects, initializes a new server struct",2,{"inputs":[{"name":"handle"},{"name":"string"},{"generics":["string"],"name":"sender"},{"generics":["mutex"],"name":"arc"}],"output":{"name":"self"}}],[11,"call","","",2,null]],"paths":[[4,"Command"],[3,"HostMap"],[3,"Server"]]};
initSearch(searchIndex);
