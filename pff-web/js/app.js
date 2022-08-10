const _ = require('lodash');
const Handlebars = require('handlebars');

const template = Handlebars.compile(
  document.getElementById('search-results-template').innerHTML
);

document.addEventListener('DOMContentLoaded', () => {
  // set focus on the search box
  const searchInput = document.getElementById('search');
  searchInput.focus();
  searchInput.addEventListener('keyup', _.debounce(search, 500));

  // during dev its nice to have a search result already in place
  search();
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
  }
}
