// Adds an option to SUMS's "show x entries" to show 100000 entries. It should
// be noted that SUMS's member dashboard doesn't actually paginate requests, but
// just annoyingly hides entries after the fact.

// Returns the value of the new option.
let newOptionValue = 100000;

let selector = document.querySelector("#group-member-list-datatable_length > label:nth-child(1) > select:nth-child(1)");

if (selector != null) {
    let newOption = document.createElement("option");
    newOption.setAttribute("value", newOptionValue);
    newOption.innerHTML = newOptionValue;

    selector.appendChild(newOption);
    selector.value = newOptionValue;
}

return newOptionValue;