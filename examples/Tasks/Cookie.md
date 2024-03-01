# Add Cookie

This task will add cookie.

## Fields (Required)
* name: A small decription of what the taks will do.
* cookie_key: cookie key that will be added 
* cookie_value: cookie value that will be added
* path: The cookie path that will be used
* domain: The cookie domain that will be used

## Example

```
- name: 'Add cookie'
  cookie: 
    cookie_key: 'Name'
    cookie_value: 'John'
    path: '/'
    domain: 'localhost'
```      

## Variables support
```
  - name: "Click search button"
    cookie: 
      cookie_key: '{cookie_key}'
      cookie_value: '{cookie_value}'
      path: '{cookie_path}'
      domain: '{cookie_domain}'
```        