# Validate Task

This task can be use to validate an element text, css and properties

## Fields 
### Required
* Name: A small decription of what the taks will do.
* element: Locating the elements based on the provided locator values
    #### Locator strategies:
    * id
    * xPath
    * className

### Optional    
* text: The expected text
* css: The CSS properties you want to validate -> `css-property: expected`
* property: the properties you want to validate -> `property: expected`

## Example
```
  - name: "Validate Title"
    validate:
      element:
        xPath: '//*[@id="firstHeading"]/span' 
      expect:
        css:
          color: 'rgb(0, 0, 0)'
          display: 'inline-block'
          overflow: 'visible'
          'text-indent': '0px'
        text: '{input}'
        property:
          name: 'foo'  
```      
