import axios from "axios";
import { Formik } from "formik";
import { useState } from "react";
import {
  Button,
  Card,
  Colors,
  Text,
  TextField,
  View,
} from "react-native-ui-lib";

export default function ChatView() {
  const [question, setQuestion] = useState<string>("");
  const [answer, setAnswer] = useState<string>("");

  const onSubmit = async (values: any) => {
    console.log(values);
    const { data } = await axios.post("http://localhost:8001/ask-question", {
      question: values.question,
    });

    setQuestion(values.question);
    setAnswer(data);
  };

  return (
    <View style={{ padding: 20 }}>
      <Card style={{ padding: 20, marginTop: 50 }}>
        <Text text40 style={{marginBottom: 10}}> Ask Me</Text>
        <Formik
          initialValues={{ question: "" }}
          onSubmit={(values) => onSubmit(values)}
        >
          {({ handleChange, handleBlur, handleSubmit, values }) => (
            <View>
              <TextField
                placeholder="How can I help you? Type here."
                text60L
                floatingPlaceholder
                value={values.question}
                onChangeText={handleChange("question")}
                onBlur={handleBlur("question")}
              />

              <Button
                label="Ask"
                style={{ marginTop: 20 }}
                text60L
                backgroundColor={Colors.green30}
                borderRadius={10}
                onPress={() => handleSubmit()}
              />
            </View>
          )}
        </Formik>

        <View style={{ marginTop: 30 }}>
          <Text text60L style={{ marginBottom: 5 }}>
            Question:
          </Text>
          <Text text70L color={Colors.blue30}>
            {question}
          </Text>

          <Text text60L style={{ marginBottom: 5 }}>
            Answer:
          </Text>
          <Text text70L color={Colors.orange30}>
            {answer}
          </Text>
        </View>
      </Card>
    </View>
  );
}
