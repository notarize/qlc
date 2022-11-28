export type GetByNodeConcreteOnly_desiredUser_User = {
  email: string;
  firstName: string;
};

export type GetByNodeConcreteOnly_desiredUser_$$other = {

};

export type GetByNodeConcreteOnly_desiredUser = GetByNodeConcreteOnly_desiredUser_User | GetByNodeConcreteOnly_desiredUser_$$other;

export type GetByNodeConcreteOnly = {
  desiredUser: GetByNodeConcreteOnly_desiredUser | null;
};
