const Handlebars = require('handlebars');

const template = Handlebars.compile(
  document.getElementById('search-results-template').innerHTML
);

document.addEventListener('DOMContentLoaded', () => {
  // set focus on the search box
  const searchInput = document.getElementById('search');
  searchInput.focus();

  // handle searching with debounce
  searchInput.addEventListener('keyup', debounce(search, 500));
});

async function search() {
  const search = document.getElementById('search').value;
  if (search.trim().length > 0) {
    const params = new URLSearchParams();
    params.append('q', search);
    const url = `/search?${params.toString()}`;
    const response = await fetch(url);
    const data = await response.json();
    const html = template(data);
    document.getElementById('search-results').innerHTML = html;

    setTimeout(() => {
      let res = document
        .querySelector('#search-results')
        .querySelectorAll('#message');
      let last = res[res.length - 1];
    }, 1);
  }
}

function debounce(callback, delay) {
  var timeout = null;
  return function() {
    //
    // if a timeout has been registered before then
    // cancel it so that we can setup a fresh timeout
    //
    if (timeout) {
      clearTimeout(timeout);
    }
    var args = arguments;
    timeout = setTimeout(function() {
      callback.apply(null, args);
      timeout = null;
    }, delay);
  };
}
