const Handlebars = require('handlebars');

const template = Handlebars.compile(
  document.getElementById('search-results-template').innerHTML
);

let searchResults = {
  messages: [],
  total_matches: 0
};
let offset = 0;
let scrollToBottom = false;

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
    params.append('offset', offset);
    const url = `/search?${params.toString()}`;
    const response = await fetch(url);
    const results = await response.json();
    searchResults.messages = searchResults.messages.concat(results.messages);
    const html = template(searchResults);
    document.getElementById('search-results').innerHTML = html;

    // handle load more
    setTimeout(() => {
      document.querySelector('#load-more').addEventListener('click', onLoadMore);
    }, 1);

    if (scrollToBottom === true) {
      // TODO: Implement this
      scrollToBottom = false;
    }
  }
}

function onLoadMore() {
  if ((offset + searchResults.messages.length) < searchResults.total_matches) {
    offset += searchResults.messages.length;

    // kick off another search
    scrollToBottom = true;
    search();
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
