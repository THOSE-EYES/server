// Learn more at https://deno.land/manual/examples/module_metadata#concepts
import * as proc from "node:child_process";

let TN = 0;
if (import.meta.main) {
  const server = proc.spawn("../target/debug/server");

  // Register user U1
  await etry("/register", undefined, {
    name: "U1",
    surname: "A",
    password: "wow",
  });
  // Register user U2
  await etry("/register", undefined, {
    name: "U2",
    surname: "B",
    password: "owo",
  });
  // Query all users
  await etry("/users", undefined, undefined);
  // Login as U1 + save session_id
  const r1 = await etry("/login", undefined, { user_id: 1, password: "wow" });
  const sid1 = r1.data.session_id;
  // Create group chat G1
  await etry("/create", { session_id: sid1 }, {
    title: "G1",
    description: "Room description",
  });
  // Query U1's chats = save id of G1
  const r2 = await etry("/chats", { session_id: sid1 }, undefined);
  const cid1 = r2.data.chats[0].id;
  // Send message 'Hello!' to G1 as U1
  await etry("/message", { session_id: sid1 }, {
    chat_id: cid1,
    content: "Hello!",
  });
  // Query messages in G1
  await etry("/messages", { session_id: sid1, chat_id: cid1 }, undefined);
  // Invite U2 to G1
  await etry("/invite", { session_id: sid1 }, { chat_id: cid1, user_id: 2 });
  // Login as U2 + save session_id
  const r3 = await etry("/login", undefined, { user_id: 2, password: "owo" });
  const sid2 = r3.data.session_id;
  // Query U2's chats
  await etry("/chats", { session_id: sid2 }, undefined);
  // Send message 'Hi :)' to G1 as U2
  await etry("/message", { session_id: sid2 }, {
    chat_id: cid1,
    content: "Hi :)",
  });
  // Query messages in G1
  await etry("/messages", { session_id: sid2, chat_id: cid1 }, undefined);
  // Query U1's activity
  await etry("/getActivity", undefined, { user_id: 1 });
  // Logout as U1
  await etry("/logout", { session_id: sid1 }, undefined);
  // Query U1's activity
  await etry("/getActivity", undefined, { user_id: 1 });
  
  server.kill("SIGTERM");
}

async function etry(endpoint, params, body) {
  TN += 1;
  try {
    const options = body
      ? {
        method: "POST",
        body: JSON.stringify(body),
        headers: { "Content-type": "application/json; charset=UTF-8" },
      }
      : { method: "GET" };
    let url = "http://127.0.0.1:3030" + endpoint;
    if (params) url += "?" + new URLSearchParams(params).toString();
    console.log("\n-----" + "# TEST-" + TN + " " + url);
    console.log(options);
    const jres = await fetch(url, options);
    let response = { ok: jres.ok, status: jres.status };
    if (jres.body) {
      try {
        response.data = await jres.json();
      } catch (e) {
        console.log(jres);
      }
    }
    console.log(response);
    return response;
  } catch (e) {
    console.log(e);
    return null;
  }
}
