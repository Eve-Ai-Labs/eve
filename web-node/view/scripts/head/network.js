import "/scripts/lib.js?b=00000000000000";


export async function init() {
  let api = await window.web_node.get_api();
  console.log('api', api);
  let networkName = document.getElementById('network-name');
  if (api.includes('stage')) {
    networkName.textContent = 'stage';
  } else if (api.includes('testnet')) {
    networkName.textContent = 'testnet';
  } else {
    networkName.textContent = 'local';
  }
}
