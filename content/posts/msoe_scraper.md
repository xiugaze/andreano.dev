---
title: "How to Schedule for Classes at MSOE"
date: 2023-05-18T09:30:38-05:00
tags: []
draft: false
summary: "Whining about class scheduling and building a web scraper"
---

{{< repo >}}
  msoe-open-seat
{{< /repo >}}

Scheduling for new classes at MSOE can be somewhat confusing and stressful. 
In this article, I want to quickly share a few tips and tricks I've learned from my time here at school.

## Scheduling

Right around week seven or eight, you'll receive your scheduling time in [MyMSOE](https://my.msoe.edu). 

Sometimes, you'll get a late scheduling time, which means you may not be able to get all of the classes you want. 
Unfortunately, you don't really have a lot of control over this process. 
Your advisor might tell you that your registration time is dependent on how many credits you have, and that people are filtered into buckets based on their academic standing. 
However, the truth is that you're probably scheduling late because of a loose cosmic ray that flipped your credit count from 60 to -127, or because Mercury is in retrograde, or because you forgot to sacrifice a goat to the registrar during week 6.

If you're really lucky, you'll get an early scheduling date. 
This means that instead of having to compete with the rest of your class to land one of eighteen available seats for a class, you get to compete with half of your class to land one of eight seats that the registrar decided to open up. 

In either case, you might try to schedule and find that one of the classes you want is full. 
If this happens, don't worry!
You have a lot of options at this point:

- File a closed section request: This form asks a department to put you in a class that's already full. This is a great way to practice coping with rejection emails from your internship applications!
- Schedule for an open section: You may have to put in some extra work to do well in a class with a more difficult professor, but hey, at the end of the day they're all fair in the end, right?
- Wait for the section to open up: Seats can *always* be added to sections. In my experience, more seats tend to be added at times ending in a prime number, like 8:07 AM or 12:67 PM.

## What happens if I don't get my schedule?

If a class is a requirement in your academic track, then the school is required to give you a seat. 
In this case, a closed section request filed when all section are full will usually be approved without issue. This can be acutally be a good thing, because an 8:00 AM to 7:00 PM schedule is actually really good because it's a realistic simulation of a real work day in industry!

If you end up with a really bad schedule, learning new skills can help you cope with the reality you're studying for finals already dreading the next term!

As of right now, I'm not feeling all that good about my schedule for the fall. 
To distract us, let's learn how to build a simple web scraper together!

## web scraping

Web scraping is a way of programatically retrieving data or content from a website. 
We can use these techniques to automatically extract the static HTML data from a website, and with it, potentially database contents that gets served alongside the static website.  

For our little application, I'm going to just pick a [random website](https://resources.msoe.edu/sched/courses/all) that we can easily pull some data off of. 

![targets](/images/scheduler.png)

We're going to have two major dependencies for our Python script: `request` from the standard library for our HTTP functions, and a library called [Beautiful Soup](https://www.crummy.com/software/BeautifulSoup/bs4/doc/) (`bs4`) for pulling elements from the DOM and packaging them as Python objects. 

```python
import requests
from bs4 import BeautifulSoup

URL = "https://resources.msoe.edu/sched/courses/all"

page = requests.get(URL)
soup = BeautifulSoup(page.content, "html.parser")

```

Our `page` object is a simple class that contains the data from an HTTP PUT, which is an HTTP primitive that sends data back in response to a GET request. 
You can read more about POST [here](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/PUT).
Specifically, this object is going to hold our HTML data in it's `content` attribute, which we're going to pass to the `BeautifulSoup()` constructor to instantiate the HTML document as an object tree. 
This constructor builds our DOM as a bunch of objects, and returns the root node which we'll store in the `soup` variable. 

Now that we have a handle to our webpage, we can traverse through the tree to target a specific piece of data. 
For this example, we can see that the main content of the webpage is stored in a table,and using inspect inspect element confirms this structure: 

```html 
<div class="table_wrapper">
  <div class="table_wrapper_inner">
    <table class="course-table">
        <thead>
        <tr>
            <th style="width: 90px">Code</th>
            <th>Title</th>
            <th>Status</th>
            <th>Instructor</th>
        </tr>
        </thead>
        <tbody>
            <tr>
                <td>ACS  1130 001 </td>
                <td>Introduction to Actuarial Science</td>
                <td>Open</td>
                <td>William M. Brummond</td>
            </tr>
            <tr>
                <td>ACS  1530 001 </td>
                <td>Financial Mathematics I</td>
                <td>Open</td>
                <td>. MA Staff</td>
            </tr>
            <!-- ...etc -->
```
Okay, so here we can see that all of this data is stored in a `<table>` with `class ="course-table"`
In HTML, the `class` attribute is *not* a unique identifier, meaning that there can be multiple tags that share the same class. 
However, this is the first occurence of this class in the document, so if there's any more, we can just grab the first one. 

We can get a handle to the table using  `b4f` method `find()`:

```python
table = soup.find(class_="course-table").find("tbody")
```
Here, we're calling the `find()` method on our entire document, searching for the first element with the `class` attribute `"course-table"`.
We're then calling `find()` again on the `<table>` tag for the first `<tbody>` tag. 

Looking at our HTML document, we can see that it's contains a bunch of `<tr>` (table row) tags, which in turn hold multiple `<td>` (table data) tags each. 
However, none of these elements specify any unique identifiers, like a `class` or `id` attribute. 

Let's write a quick function to iterate through the `<tbody>` tag and search for the `<tr>` that contains a `<td>` with specific text:

```python
def find_td_in_tbody(tbody, string):
  for tr in tbody.find_all('tr'):
    for td in tr.find_all('td'):
      if string in td_tag.get_text():
        return td_tag
  return None

classcode = find_td_in_body(table, "ACS 1530 001")
```

In this function, we iterate through each `<tr>` tag in the `<tbody>` tag. 
Within each of those, we iterate through each `<td>` tag, and check if it contains our search string. If it does, we return the `<td>` element, which corresponds to a cell in the table.

Now that we and element in the row containing our search string, we can check if any of its siblings contain another search string. 

```python
def find_in_sibling(tag, string):
  for sibling in tag.find_next_siblings():
    if string in sibling.get_text():
      return True
  return False
status = find_in_sibling(classcode, "Open")
```
Putting all of this together, we can create a simple program that monitors for a state change in an HTML tag:

```python
import requests
from bs4 import BeautifulSoup

def find_td_in_tbody(tbody, string):
  for tr in tbody.find_all('tr'):
    for td in tr.find_all('td'):
      if string in td_tag.get_text():
        return td_tag

def find_in_sibling(tag, string):
  for sibling in tag.find_next_siblings():
    if string in sibling.get_text():
      return True
  return False

def main():
  URL = "https://resources.msoe.edu/sched/courses/all"
  class_name = "ACS 1130 001"

  page = requests.get(URL)
  soup = BeautifulSoup(page.content, "html.parser")

  table = soup.find(class_="course-table").find("tbody")

  class_element = find_td_in_body(table, class_name)
  open = find_in_sibling(class_element, "Open")

  status: str
  if(open):
    status = "Open"
  else:
    status = "Closed"

  print("Class " + classcode + " is currently " + status)
  

if __name__ == "__main__":
  main()

```
Wow, look at that! 
We just automated the process of refreshing a page and seeing if something changed!

## conclusion

take the power back
