# Grouping is a feature in dx style where instead of using different instanches of properties for different values you use ```()``` to group them together

For example:

```html(wrong)
<div dx-text="lg:bg-red-500 lg:text-yellow-500"></div>
```

```html(correct)
<div dx-text="lg(bg-red-500 text-yellow-500)"></div>
```

Any property that has that has similar starting properties can be grouped together.
This makes the code cleaner and easier to read plus it reduces the size of the code.
Now, grouping is really a powerful feature and can be used in many ways.
And you can use grouping to create your custom classes that more straightforward than tailwindcss @apply directive, as you don't have to move to css file and write a code that is not actually a css code and it will provide a log about what class has been uses in that custom class. Like this:

```html(tailwindcss - as you have to move to css file and write code that is code that really to see first to know what properties has been used - this is not a problem in react, nextjs and other frameworks that uses components but still at what if you just want to use just styles of that component not the component itself)
<div dx-text="card"></div>
```

```html(correct)
<div dx-text="card(bg-red-500 h-50 w-full text-yellow-500+)"></div> -> when you are using + it means you can add more properties to that class
<div dx-text="card(BHWT+)"></div> -> the classes will be converter to to the first letter of each property after saving
<div dx-text="card"></div> -> and then you can just remove the () sign if you want or just keep it as it is
And as you can see there option if want to put all the styles into one custom class or just keep a recored in dx style of that class list for later usage and inside this grouping symbol you can also use jus the first letter of each property to make it even shorter.
And again as the @ sign is just most useful feature where you can use other people's code in your own code, you can also use grouping with it like this:

```html
<div dx-text="@essencefromexistnece:card"></div> -> this will use the card as it is from the essencefromexistnece user
<div dx-text="@essencefromexistnece:card"></div> -> or just if you want to see what properties has been used in that class and then use utilites instead of a custom class then you again just put () and dx style just put all the properties that has been used in that class like this: <div dx-text="@essencefromexistnece:card(bg-red-500 h-50 w-full text-yellow-500)"></div>
<div dx-text="@essencefromexistnece:card(bg-red-500+)"></div> -> this will use the card from the essencefromexistnece user and add more properties to it or use - sign to remove properties from it
And if you don't want to see first letter of properties you can tell dx to write comment to html file showing all the properties that has been used in that class like this:
```<div dx-text="card(BHWT+) #bg-red-500 h-50 w-full text-yellow-500"></div> -> this will generate a comment in the html file showing all the properties that has been used in that class

```

! -> this means important

@ -> this is a very powerful feature that allows you to use other people's code in your own code.

# Hashtags -> this is a comment + it will tell dx style to generate styles into a single class as the user wants it to be

$ -> this is a variable

% -> this is a percentage

^ -> this is a power

& -> this is a reference

* -> this is a wildcard

* -> this is one of the most powerful feature, it allows you to add new properties to refrenced or other people's code.

= -> this is used to be as a logical if-else for the properties.

| => this is used as an or operator

: -> this is for property value separation

. -> this is for nested properties

< -> this is for less than

> -> this is for greater than

/ -> this is for division

? -> this is for ternary operations

~ -> this is for bitwise NOT and much more

## Status

* Supported: grouping expansion `prefix(a b)` → `prefix:a`, `prefix:b` when scanning classes.
* Supported: nested groups (balanced parentheses) and multiple chained prefixes like `lg(md(bg-red-500))`.
* Supported: scanning both `class` and `dx-*` attributes; inline `#...` comments are ignored; trailing `+` is ignored during extraction.
* Not yet: rewriting HTML to inject expanded classes, external references `@user:component(...)`, `-` removals, and `card(...)` registry/logging.
* Not yet: abbreviation compression/expansion inside grouping (e.g., `BHWT`) and emitting inline comment mirrors into HTML.

Some unused symbols are reserved:

```txt
(

)

_

-

{

}

[

]

\

;

`

'

"

,

```

These symbols are reserved for future use and will not be used in the current version of dx style.
