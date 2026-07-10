pub struct NixModule {


    pub packages: Vec<String>,


}



impl NixModule {


    pub fn generate(&self) -> String {


        let mut output =
            String::new();


        output.push_str(
            "environment.systemPackages = with pkgs; [\n"
        );


        for package in &self.packages {

            output.push_str(
                &format!(
                    "  {}\n",
                    package
                )
            );

        }


        output.push_str(
            "];\n"
        );


        output

    }

}