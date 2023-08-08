import os, sys, json
os.environ["PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION"] = "python"

from push_receiver import PushReceiver, register

# SENDER_ID = 976529667804
SENDER_ID = 138768135723
APP_ID = "1:138768135723:android:d1ad4ee253296fef039c6e"

def on_notification(obj, notification, data_message):
  idstr = data_message.persistent_id + "\n"

  # check if we already received the notification
  with open("persistent_ids.txt", "r") as f:
    if idstr in f:
      return

  # new notification, store id so we don't read it again
  with open("persistent_ids.txt", "a") as f:
    f.write(idstr)

  # print notification
  n = notification["notification"]
  text = n["title"]
  if n["body"]:
    text += ": " + n["body"]
  print(text)


if __name__ == "__main__":
  task = str(sys.argv[1]).rstrip()

  creds_file = str(sys.argv[2]).rstrip()

  if task == "reg":
    credentials = register.register(sender_id=SENDER_ID, app_id=APP_ID)
    with open(creds_file, "w") as f:
      json.dump(credentials, f)

    print("ID:{}".format(credentials["fcm"]["token"]))
  
  else:
    with open(creds_file, "r") as f:
      credentials = json.load(f)

    with open("../persistent_ids.txt", "a+") as f:
      received_persistent_ids = [x.strip() for x in f]

    print("Listening")
    receiver = PushReceiver(credentials, received_persistent_ids)
    receiver.listen(on_notification)
    # listen(credentials, on_notification, received_persistent_ids)
