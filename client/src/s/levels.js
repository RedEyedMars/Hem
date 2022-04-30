import testA  from '../../../../res/levels/test_a.lvl';
import * as golems from '../../../../golems.js';

export async function fetch_level(filename) {
  console.log("get");
  const response = await fetch(`/l/${filename}`, {
    method: 'get',
    credentials: 'same-origin', // include, *same-origin, omit
    headers: {
      'Content-Type': 'application/json'
    },
    redirect: 'follow', // manual, *follow, error
    referrerPolicy: 'no-referrer',
  });
  return response.text();
}

export function post_level(filename, body) {
  console.log("post");
  fetch("/l", {
    method: 'POST',
    credentials: 'same-origin', // include, *same-origin, omit
    headers: {
      'Content-Type': 'application/json'
    },
    redirect: 'follow', // manual, *follow, error
    referrerPolicy: 'no-referrer',
    body: JSON.stringify({
      filename,
      body
    })
  }).catch(error => {
    console.log("Error occurred while saving level");
    console.log(error);
  });
}
