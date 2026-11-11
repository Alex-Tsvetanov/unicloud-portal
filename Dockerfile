# Build stage: package the app, then build a Java runtime that holds only the modules it uses.
# Alpine (musl) on both stages, because a jlink runtime only runs on the libc it was built for.
FROM eclipse-temurin:25-jdk-alpine AS build
WORKDIR /src
COPY .mvn/ .mvn/
COPY mvnw pom.xml ./
RUN chmod +x mvnw && ./mvnw -B -q dependency:go-offline
COPY src/ src/
RUN ./mvnw -B -q -DskipTests package && cp target/*.jar app.jar

# Split the fat jar into Spring Boot's layers (dependencies change rarely, application on every
# commit), so each deploy pushes and pulls only the small application layer.
RUN java -Djarmode=tools -jar app.jar extract --layers --destination /layers

# Modules found by jdeps, plus ones looked up by name at runtime, which jdeps can't see:
# jdk.charsets (extra charsets for the MySQL connector), jdk.zipfs and jdk.management.
RUN MODULES="$(jdeps --ignore-missing-deps -q --recursive --multi-release 25 --print-module-deps \
        --class-path '/layers/dependencies/lib/*' /layers/application/app.jar)" \
 && echo "jdeps modules: $MODULES" \
 && jlink --add-modules "$MODULES,jdk.charsets,jdk.zipfs,jdk.management" \
        --strip-debug --no-man-pages --no-header-files --compress=zip-6 \
        --output /jre

# Run stage: plain Alpine, the trimmed runtime and the app layers
FROM alpine:3.23
ENV JAVA_HOME=/opt/java
ENV PATH="$JAVA_HOME/bin:$PATH"
COPY --from=build /jre $JAVA_HOME
WORKDIR /app
COPY --from=build /layers/dependencies/ ./
COPY --from=build /layers/spring-boot-loader/ ./
COPY --from=build /layers/snapshot-dependencies/ ./
COPY --from=build /layers/application/ ./
ARG GIT_SHA=dev
ENV GIT_SHA=$GIT_SHA
EXPOSE 8080
# Numeric uid:gid; Alpine has no user 1000, so without the gid the process would run in group root
USER 1000:1000
ENTRYPOINT ["java", "-XX:MaxRAMPercentage=75", "-jar", "app.jar"]
