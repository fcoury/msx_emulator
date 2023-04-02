export async function fetchProgram() {
  return fetch("/program")
    .then((response) => response.json())
    .then((data) => lemonade.dispatch("Program", data));
}
