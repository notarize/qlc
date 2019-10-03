use crate::helpers::basic_success_assert_typescript;

#[test]
fn compile_simple_query() {
    basic_success_assert_typescript(
r"
const query = gql`
    query TestQuery {
        viewer {
            id
            me: user {
                id
            }
        }
    }
`;
",
    "TestQuery.ts",
    "
export interface TestQuery_viewer_me {
  id: string;
}

export interface TestQuery_viewer {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  me: TestQuery_viewer_me | null;
}

export interface TestQuery {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
}
    ")
}
